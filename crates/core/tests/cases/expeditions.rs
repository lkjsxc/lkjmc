#[sqlx::test(migrations = "../../migrations")]
async fn expedition_journal_is_retained_bounded_and_participant_scoped(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Expedition owner", false).await;
    let participant = account(&app, "Expedition participant", false).await;
    let outsider = account(&app, "Expedition outsider", false).await;
    for index in 0..55 {
        let expedition = Uuid::new_v4();
        sqlx::query("INSERT INTO adventures(id,owner,state,created_at) VALUES($1,$2,'closed','2026-10-04T00:00:00Z'::timestamptz+$3*interval '1 minute')")
            .bind(expedition).bind(owner.id).bind(index).execute(&app.db).await.unwrap();
        for actor in [&owner, &participant] {
            sqlx::query("INSERT INTO adventure_participants(adventure_id,account_id,released_at) VALUES($1,$2,now())")
                .bind(expedition).bind(actor.id).execute(&app.db).await.unwrap();
        }
    }
    let mut db = app.db.acquire().await.unwrap();
    let mut seen = std::collections::BTreeSet::new();
    let mut before = None;
    loop {
        let page = lkjmc_core::expeditions::rows(&mut db, participant.id, 25, before, None)
            .await
            .unwrap();
        let rows = page.as_array().unwrap();
        assert!(rows.len() <= 25);
        for row in rows {
            assert!(seen.insert(row["id"].as_str().unwrap().to_string()));
            assert_eq!(row["participants"].as_array().unwrap().len(), 2);
            assert_eq!(row["lifetime"], "temporary");
            assert_eq!(row["destination"], "end");
            assert_eq!(row["can_enter"], false);
        }
        let Some(last) = rows.last() else {
            break;
        };
        before = Some((
            last["created_at"].as_str().unwrap().parse().unwrap(),
            id(last, "id"),
        ));
    }
    assert_eq!(seen.len(), 55);
    let result = lkjmc_core::expeditions::rows(&mut db, outsider.id, 100, None, None)
        .await
        .unwrap();
    assert!(result.as_array().unwrap().is_empty());
    let retained = seen.iter().next().unwrap().parse().unwrap();
    assert!(
        lkjmc_core::expeditions::rows(&mut db, outsider.id, 1, None, Some(retained))
            .await
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn expedition_preparation_waits_for_the_readiness_snapshot(pool: PgPool) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    let owner = account(&app, "Ready owner", false).await;
    let member = account(&app, "Ready member", false).await;
    for actor in [&owner, &member] {
        fund(&app, actor, 3000).await;
        sqlx::query("INSERT INTO game_sessions(account_id,profile_id,native_uuid,session_id,server_id,lease_until) SELECT $1,id,$2,$3,$4,now()+interval '5 minutes' FROM profiles WHERE account_id=$1 AND status='active'")
            .bind(actor.id).bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(server).execute(&app.db).await.unwrap();
    }
    let party = id(
        &run(
            &app,
            &owner,
            Command::PartyCreate {
                name: "Consent".into(),
            },
        )
        .await,
        "party_id",
    );
    let invitation = run(
        &app,
        &owner,
        Command::Invite {
            kind: "party".into(),
            resource: party,
            target: member.id,
        },
    )
    .await;
    run(
        &app,
        &member,
        Command::InviteRespond {
            id: id(&invitation, "id"),
            accept: true,
        },
    )
    .await;
    for actor in [&owner, &member] {
        run(&app, actor, Command::PartyReady { party: None, ready: true }).await;
    }
    let mut changing = app.db.begin().await.unwrap();
    sqlx::query("UPDATE party_members SET ready=false WHERE account_id=$1")
        .bind(member.id)
        .execute(&mut *changing)
        .await
        .unwrap();
    let preparing_app = app.clone();
    let preparing_owner = owner.clone();
    let mut preparing = tokio::spawn(async move {
        commands::execute(
            &preparing_app,
            &preparing_owner,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::ExpeditionPrepare,
            },
        )
        .await
    });
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(200), &mut preparing)
            .await
            .is_err(),
        "Preparation must wait for an in-flight readiness change before committing consent"
    );
    changing.commit().await.unwrap();
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), preparing)
        .await
        .unwrap()
        .unwrap();
    assert!(result.is_err());
    let reserved: i64 = sqlx::query_scalar("SELECT reserved FROM wallets WHERE owner=$1")
        .bind(owner.id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(reserved, 0);
    let runs: i64 = sqlx::query_scalar("SELECT count(*) FROM adventures WHERE owner=$1")
        .bind(owner.id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(runs, 0);
}

#[sqlx::test(migrations = false)]
async fn expedition_upgrade_preserves_active_roster_and_receipt_identity(pool: PgPool) {
    for migration in sqlx::migrate!("../../migrations")
        .iter()
        .filter(|m| m.version < 18)
    {
        sqlx::raw_sql(&migration.sql).execute(&pool).await.unwrap();
    }
    let app = app(pool);
    let (server, _) = official(&app).await;
    let owner = account(&app, "Existing expedition", false).await;
    let world = Uuid::new_v4();
    let native = Uuid::new_v4();
    let expedition = Uuid::new_v4();
    let receipt = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO worlds(id,server_id,name,kind,native_uuid) VALUES($1,$2,$3,'private_end',$4)",
    )
    .bind(world)
    .bind(server)
    .bind(format!("adventure_{expedition}"))
    .bind(native)
    .execute(&app.db)
    .await
    .unwrap();
    sqlx::query("INSERT INTO adventures(id,owner,world_id,state,opens_at,expires_at) VALUES($1,$2,$3,'active',now(),now()+interval '3 hours')")
        .bind(expedition).bind(owner.id).bind(world).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO adventure_participants(adventure_id,account_id) VALUES($1,$2)")
        .bind(expedition)
        .bind(owner.id)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO jobs(id,actor,server_id,worker,kind,payload,state,result) VALUES($1,$2,$3,'official','adventure.prepare',$4,'succeeded',$5)")
        .bind(receipt).bind(owner.id).bind(server).bind(json!({"adventure_id":expedition})).bind(json!({"effect":"committed","receipt_hash":"immutable"})).execute(&app.db).await.unwrap();
    sqlx::raw_sql(include_str!("../../../../migrations/0018_expeditions.sql"))
        .execute(&app.db)
        .await
        .unwrap();
    let consent: bool = sqlx::query_scalar("SELECT consented_at IS NOT NULL AND committed_at IS NOT NULL AND released_at IS NULL FROM adventure_participants WHERE adventure_id=$1")
        .bind(expedition).fetch_one(&app.db).await.unwrap();
    assert!(consent);
    let metadata: Value = sqlx::query_scalar("SELECT jsonb_build_object('native_uuid',native_uuid,'lifetime',lifetime,'environment',environment,'access_policy',access_policy) FROM worlds WHERE id=$1")
        .bind(world).fetch_one(&app.db).await.unwrap();
    assert_eq!(
        metadata,
        json!({"native_uuid":native,"lifetime":"temporary","environment":"end","access_policy":"participants"})
    );
    let unchanged: Value = sqlx::query_scalar("SELECT jsonb_build_object('kind',kind,'payload',payload,'result',result) FROM jobs WHERE id=$1")
        .bind(receipt).fetch_one(&app.db).await.unwrap();
    assert_eq!(
        unchanged,
        json!({"kind":"adventure.prepare","payload":{"adventure_id":expedition},"result":{"effect":"committed","receipt_hash":"immutable"}})
    );
}
