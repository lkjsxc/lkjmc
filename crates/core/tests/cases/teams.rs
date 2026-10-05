async fn multi_team_command(app: &App, actor: &Actor, command: Command) -> (StatusCode, Value) {
    http(
        app,
        actor,
        "POST",
        "/api/v1/commands",
        json!({"request_id":Uuid::new_v4(),"command":command}),
        true,
    )
    .await
}

async fn multi_team_ok(app: &App, actor: &Actor, command: Command) -> Value {
    let (status, response) = multi_team_command(app, actor, command).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    response["result"].clone()
}

async fn multi_team_create(app: &App, leader: &Actor, name: &str) -> (Uuid, Uuid) {
    let result = multi_team_ok(app, leader, Command::TeamCreate { name: name.into() }).await;
    (id(&result, "team_id"), id(&result, "room_id"))
}

async fn multi_team_join(app: &App, leader: &Actor, member: &Actor, team: Uuid) {
    let invitation = multi_team_ok(
        app,
        leader,
        Command::Invite {
            kind: "team".into(),
            resource: team,
            target: member.id,
        },
    )
    .await;
    multi_team_ok(
        app,
        member,
        Command::InviteRespond {
            id: id(&invitation, "id"),
            accept: true,
        },
    )
    .await;
}

async fn multi_team_permissions(
    app: &App,
    leader: &Actor,
    member: &Actor,
    team: Uuid,
    flags: [bool; 5],
) {
    let [build, sell, spend, members, administer] = flags;
    multi_team_ok(
        app,
        leader,
        Command::TeamPermissions {
            team,
            member: member.id,
            build,
            sell,
            spend,
            members,
            administer,
        },
    )
    .await;
}

async fn multi_team_detail(app: &App, viewer: &Actor, team: Uuid) -> Value {
    let (status, response) = http(
        app,
        viewer,
        "GET",
        &format!("/api/v1/teams/{team}"),
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{response}");
    response
}

async fn multi_team_land(app: &App, owner: Uuid) -> i32 {
    sqlx::query_scalar("SELECT chunks FROM land_allowances WHERE owner=$1")
        .bind(owner)
        .fetch_one(&app.db)
        .await
        .unwrap()
}

async fn multi_team_balance(app: &App, owner: Uuid) -> i64 {
    sqlx::query_scalar("SELECT balance FROM wallets WHERE owner=$1")
        .bind(owner)
        .fetch_one(&app.db)
        .await
        .unwrap()
}

async fn multi_team_progress(app: &App, owner: Uuid, key: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT progress FROM achievement_progress WHERE owner=$1 AND achievement=$2",
    )
    .bind(owner)
    .bind(key)
    .fetch_optional(&app.db)
    .await
    .unwrap()
    .unwrap_or(0)
}

async fn multi_team_rules(app: &App, target: i64) {
    sqlx::query("INSERT INTO achievements(key,title,description,event,target,land_chunks,coins,team) VALUES('multi_personal','Personal fixture','Test verified placement progress','block.placed',$1,1,7,false),('multi_shared','Shared fixture','Test selected team progress','block.placed',$1,3,11,true)")
        .bind(target).execute(&app.db).await.unwrap();
}

async fn multi_team_event_session(app: &App, actor: &Actor, server: Uuid) -> Uuid {
    // Equivalent to an adapter-verified recent connection, without launching a JVM.
    let session = Uuid::new_v4();
    sqlx::query("INSERT INTO game_session_history(session_id,server_id,account_id,profile_id) SELECT $1,$2,$3,id FROM profiles WHERE account_id=$3 AND status='active'")
        .bind(session).bind(server).bind(actor.id).execute(&app.db).await.unwrap();
    session
}

fn multi_team_event(actor: &Actor, session: Uuid, event: Uuid, amount: i64) -> Value {
    json!({"id":event,"account_id":actor.id,"session_id":session,"occurred_at":chrono::Utc::now(),"kind":"block.placed","payload":{"amount":amount}})
}

async fn multi_team_record(app: &App, server: Uuid, event: Value) -> Value {
    let (status, response) = internal(
        app,
        "official",
        Some(server),
        "/internal/v1/game/event",
        event,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{response}");
    response
}

async fn multi_team_recipient(app: &App, event: Uuid) -> Option<Uuid> {
    sqlx::query_scalar("SELECT contribution_team_id FROM game_events WHERE id=$1")
        .bind(event)
        .fetch_one(&app.db)
        .await
        .unwrap()
}

async fn multi_team_wait_lock(app: &App, table: &str, lock: &str) {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let waiting: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE datname=current_database() AND wait_event_type='Lock' AND query LIKE $1 AND query LIKE $2)")
                .bind(format!("%FROM {table}%"))
                .bind(format!("%FOR {lock}%"))
                .fetch_one(&app.db).await.unwrap();
            if waiting { break; }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }).await.unwrap_or_else(|_| panic!("Request did not reach {table} FOR {lock}"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn multi_team_acceptance_serializes_selection_and_membership_changes(pool: PgPool) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    let leader_a = account(&app, "Ordered A leader", false).await;
    let leader_b = account(&app, "Ordered B leader", false).await;
    let member = account(&app, "Ordered contributor", false).await;
    let (team_a, _) = multi_team_create(&app, &leader_a, "Ordered A").await;
    let (team_b, _) = multi_team_create(&app, &leader_b, "Ordered B").await;
    multi_team_join(&app, &leader_a, &member, team_a).await;
    multi_team_join(&app, &leader_b, &member, team_b).await;
    multi_team_rules(&app, 10).await;
    let session = multi_team_event_session(&app, &member, server).await;

    for (index, selected, change) in [
        (
            0,
            team_a,
            Command::TeamContributionSet { team: Some(team_b) },
        ),
        (1, team_b, Command::TeamLeave { team: team_b }),
    ] {
        let mut gate = app.db.begin().await.unwrap();
        sqlx::query("SELECT id FROM teams WHERE id=$1 FOR UPDATE")
            .bind(selected)
            .fetch_one(&mut *gate)
            .await
            .unwrap();
        let event = Uuid::new_v4();
        let body = multi_team_event(&member, session, event, 2);
        let accepting_app = app.clone();
        let accepting_body = body.clone();
        let accepting = tokio::spawn(async move {
            internal(
                &accepting_app,
                "official",
                Some(server),
                "/internal/v1/game/event",
                accepting_body,
            )
            .await
        });
        // Acceptance has taken the actor lock and is now waiting on the selected
        // team. A switch/leave must wait behind this transaction, not redirect it.
        multi_team_wait_lock(&app, "teams", "SHARE").await;
        let changing_app = app.clone();
        let changing_actor = member.clone();
        let changing = tokio::spawn(async move {
            commands::execute(
                &changing_app,
                &changing_actor,
                Request {
                    request_id: Uuid::new_v4(),
                    command: change,
                },
            )
            .await
        });
        multi_team_wait_lock(&app, "accounts", "UPDATE").await;
        gate.commit().await.unwrap();
        let (status, response) = tokio::time::timeout(std::time::Duration::from_secs(5), accepting)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(status, StatusCode::OK, "{response}");
        assert!(
            tokio::time::timeout(std::time::Duration::from_secs(5), changing)
                .await
                .unwrap()
                .unwrap()
                .is_ok()
        );
        assert_eq!(multi_team_recipient(&app, event).await, Some(selected));
        assert_eq!(
            multi_team_progress(&app, member.id, "multi_personal").await,
            2 * (index + 1)
        );
        assert_eq!(multi_team_progress(&app, team_a, "multi_shared").await, 2);
        assert_eq!(
            multi_team_progress(&app, team_b, "multi_shared").await,
            2 * index
        );
        assert_eq!(
            multi_team_record(&app, server, body).await["duplicate"],
            true
        );
        assert_eq!(
            multi_team_progress(&app, member.id, "multi_personal").await,
            2 * (index + 1)
        );
    }
    let after_leave = Uuid::new_v4();
    multi_team_record(
        &app,
        server,
        multi_team_event(&member, session, after_leave, 1),
    )
    .await;
    assert_eq!(multi_team_recipient(&app, after_leave).await, None);
    assert_eq!(
        multi_team_progress(&app, member.id, "multi_personal").await,
        5
    );
    assert_eq!(multi_team_progress(&app, team_a, "multi_shared").await, 2);
    assert_eq!(multi_team_progress(&app, team_b, "multi_shared").await, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn multi_team_permissions_duplicate_invites_and_leave_are_scoped(pool: PgPool) {
    let app = app(pool);
    let leader_a = account(&app, "Builders leader", false).await;
    let leader_b = account(&app, "Traders leader", false).await;
    let member = account(&app, "Multi-team member", false).await;
    let outsider = account(&app, "Outsider", false).await;
    let (team_a, room_a) = multi_team_create(&app, &leader_a, "Builders").await;
    let (team_b, room_b) = multi_team_create(&app, &leader_b, "Traders").await;
    for team in [team_a, team_b] {
        assert_eq!(multi_team_land(&app, team).await, 0);
    }
    multi_team_join(&app, &leader_a, &member, team_a).await;
    multi_team_join(&app, &leader_b, &member, team_b).await;
    multi_team_permissions(
        &app,
        &leader_a,
        &member,
        team_a,
        [true, true, false, false, false],
    )
    .await;
    multi_team_permissions(
        &app,
        &leader_b,
        &member,
        team_b,
        [false, false, true, false, false],
    )
    .await;
    let (owned, owned_room) = multi_team_create(&app, &member, "Member's own team").await;
    assert_eq!(multi_team_land(&app, owned).await, 0);
    let (status, social) = http(
        &app,
        &member,
        "GET",
        "/api/v1/view/social?section=teams",
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{social}");
    assert_eq!(social["teams"].as_array().unwrap().len(), 3);
    assert_eq!(
        social["contribution_team_id"],
        json!(team_a),
        "Joining and creating more teams must preserve the existing choice"
    );
    let a = multi_team_detail(&app, &member, team_a).await;
    let b = multi_team_detail(&app, &member, team_b).await;
    assert_eq!(a["team"]["permissions"]["can_sell"], true);
    assert_eq!(a["team"]["permissions"]["can_spend"], false);
    assert_eq!(b["team"]["permissions"]["can_build"], false);
    assert_eq!(b["team"]["permissions"]["can_spend"], true);
    assert_eq!(
        multi_team_detail(&app, &member, owned).await["team"]["permissions"]["can_administer"],
        true
    );
    assert_eq!(
        http(
            &app,
            &outsider,
            "GET",
            &format!("/api/v1/teams/{team_b}"),
            json!({}),
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );

    let mut tx = app.db.begin().await.unwrap();
    economy::book(
        &mut tx,
        leader_b.id,
        &format!("multi-team-fixture:{}", Uuid::new_v4()),
        "fixture",
        json!({}),
        &[(team_b, 20)],
        true,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        multi_team_command(
            &app,
            &member,
            Command::WalletTransfer {
                owner: Some(team_a),
                target: member.id,
                amount: 5
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    multi_team_ok(
        &app,
        &member,
        Command::WalletTransfer {
            owner: Some(team_b),
            target: member.id,
            amount: 5,
        },
    )
    .await;
    assert_eq!(multi_team_balance(&app, team_b).await, 15);
    assert_eq!(multi_team_balance(&app, member.id).await, 5);

    multi_team_ok(&app, &member, Command::TeamContributionSet { team: None }).await;
    multi_team_join(&app, &leader_b, &member, team_b).await;
    let duplicate = multi_team_detail(&app, &member, team_b).await;
    let raw = duplicate["team"]["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["account_id"] == json!(member.id))
        .unwrap();
    assert_eq!(raw["can_build"], false);
    assert_eq!(
        raw["can_spend"], true,
        "Duplicate acceptance must not reset granted spending rights"
    );
    assert!(
        duplicate["contribution_team_id"].is_null(),
        "An existing membership must not override personal-only contributions"
    );
    assert_eq!(duplicate["team"]["member_count"], 2);
    multi_team_ok(&app, &member, Command::TeamLeave { team: team_a }).await;
    assert_eq!(
        multi_team_command(
            &app,
            &member,
            Command::MessageSend {
                room: room_a,
                body: "No longer a member".into()
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    for room in [room_b, owned_room] {
        multi_team_ok(
            &app,
            &member,
            Command::MessageSend {
                room,
                body: "Still a member here".into(),
            },
        )
        .await;
    }
    let rooms: Vec<Uuid> = sqlx::query_scalar(
        "SELECT room_id FROM room_members WHERE account_id=$1 AND room_id=ANY($2) ORDER BY room_id",
    )
    .bind(member.id)
    .bind(vec![room_a, room_b, owned_room])
    .fetch_all(&app.db)
    .await
    .unwrap();
    assert_eq!(rooms.len(), 2);
    assert!(!rooms.contains(&room_a));
    assert_eq!(
        http(
            &app,
            &member,
            "GET",
            &format!("/api/v1/teams/{team_a}"),
            json!({}),
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        multi_team_detail(&app, &member, team_b).await["team"]["permissions"]["can_spend"],
        true
    );
    assert!(
        multi_team_command(&app, &member, Command::TeamLeave { team: owned })
            .await
            .0
            .is_client_error(),
        "Leadership protection remains scoped to the owned team"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn multi_team_contributions_snapshot_one_recipient_and_retry_without_reassignment(
    pool: PgPool,
) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    let leader_a = account(&app, "A leader", false).await;
    let leader_b = account(&app, "B leader", false).await;
    let member = account(&app, "Contributor", false).await;
    let (team_a, _) = multi_team_create(&app, &leader_a, "A").await;
    let (team_b, _) = multi_team_create(&app, &leader_b, "B").await;
    multi_team_join(&app, &leader_a, &member, team_a).await;
    multi_team_join(&app, &leader_b, &member, team_b).await;
    multi_team_rules(&app, 10).await;
    let session = multi_team_event_session(&app, &member, server).await;
    let event_a = Uuid::new_v4();
    let first = multi_team_event(&member, session, event_a, 2);
    multi_team_record(&app, server, first.clone()).await;
    assert_eq!(multi_team_recipient(&app, event_a).await, Some(team_a));
    assert_eq!(
        multi_team_progress(&app, member.id, "multi_personal").await,
        2
    );
    assert_eq!(multi_team_progress(&app, team_a, "multi_shared").await, 2);
    assert_eq!(multi_team_progress(&app, team_b, "multi_shared").await, 0);
    multi_team_ok(
        &app,
        &member,
        Command::TeamContributionSet { team: Some(team_b) },
    )
    .await;
    assert_eq!(
        multi_team_record(&app, server, first.clone()).await["duplicate"],
        true
    );
    assert_eq!(multi_team_recipient(&app, event_a).await, Some(team_a));
    let event_b = Uuid::new_v4();
    multi_team_record(&app, server, multi_team_event(&member, session, event_b, 3)).await;
    assert_eq!(multi_team_recipient(&app, event_b).await, Some(team_b));
    assert_eq!(
        multi_team_progress(&app, member.id, "multi_personal").await,
        5
    );
    assert_eq!(multi_team_progress(&app, team_a, "multi_shared").await, 2);
    assert_eq!(multi_team_progress(&app, team_b, "multi_shared").await, 3);
    multi_team_ok(&app, &member, Command::TeamContributionSet { team: None }).await;
    let personal = Uuid::new_v4();
    multi_team_record(
        &app,
        server,
        multi_team_event(&member, session, personal, 1),
    )
    .await;
    assert_eq!(multi_team_recipient(&app, personal).await, None);
    assert_eq!(
        multi_team_progress(&app, member.id, "multi_personal").await,
        6
    );
    multi_team_ok(
        &app,
        &member,
        Command::TeamContributionSet { team: Some(team_b) },
    )
    .await;
    multi_team_ok(&app, &member, Command::TeamLeave { team: team_b }).await;
    let after_leave = Uuid::new_v4();
    let personal_land = multi_team_land(&app, member.id).await;
    multi_team_record(
        &app,
        server,
        multi_team_event(&member, session, after_leave, 4),
    )
    .await;
    assert_eq!(
        multi_team_recipient(&app, after_leave).await,
        None,
        "Losing the selected membership must not credit another membership"
    );
    assert_eq!(
        multi_team_progress(&app, member.id, "multi_personal").await,
        10
    );
    assert_eq!(multi_team_balance(&app, member.id).await, 7);
    assert_eq!(multi_team_land(&app, member.id).await, personal_land + 1);
    assert_eq!(multi_team_progress(&app, team_a, "multi_shared").await, 2);
    assert_eq!(multi_team_progress(&app, team_b, "multi_shared").await, 3);
    assert_eq!(
        multi_team_record(&app, server, first).await["duplicate"],
        true
    );
    multi_team_ok(
        &app,
        &member,
        Command::TeamContributionSet { team: Some(team_a) },
    )
    .await;
    let finish = multi_team_event(&member, session, Uuid::new_v4(), 8);
    multi_team_record(&app, server, finish.clone()).await;
    assert_eq!(
        multi_team_record(&app, server, finish).await["duplicate"],
        true
    );
    assert_eq!(multi_team_balance(&app, team_a).await, 11);
    assert_eq!(multi_team_land(&app, team_a).await, 3);
    assert_eq!(multi_team_balance(&app, team_b).await, 0);
    assert_eq!(multi_team_land(&app, team_b).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn multi_team_concurrent_conflicting_event_ids_mint_once(pool: PgPool) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    let member = account(&app, "Concurrent contributor", false).await;
    let (team, _) = multi_team_create(&app, &member, "Concurrent team").await;
    multi_team_rules(&app, 1).await;
    let session = multi_team_event_session(&app, &member, server).await;
    let event = Uuid::new_v4();
    let mut blocker = app.db.begin().await.unwrap();
    sqlx::query("SELECT id FROM accounts WHERE id=$1 FOR UPDATE")
        .bind(member.id)
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let mut pending = Vec::new();
    for amount in [1, 2] {
        let app = app.clone();
        let body = multi_team_event(&member, session, event, amount);
        pending.push(tokio::spawn(async move {
            internal(
                &app,
                "official",
                Some(server),
                "/internal/v1/game/event",
                body,
            )
            .await
        }));
    }
    // Both requests must have passed the initial duplicate read and be blocked
    // on the same actor. This exercises the INSERT conflict path deterministically.
    tokio::time::timeout(std::time::Duration::from_secs(5),async {
        loop {
            let waiting:i64=sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname=current_database() AND wait_event_type='Lock' AND query LIKE '%accounts%' AND query LIKE '%FOR UPDATE%'")
                .fetch_one(&app.db).await.unwrap();
            if waiting>=2 {break;}
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }).await.expect("Both conflicting events should wait on the actor lock");
    blocker.commit().await.unwrap();
    let first = pending.remove(0).await.unwrap();
    let second = pending.remove(0).await.unwrap();
    assert!(
        (first.0 == StatusCode::OK && second.0 == StatusCode::CONFLICT)
            || (second.0 == StatusCode::OK && first.0 == StatusCode::CONFLICT),
        "{first:?}; {second:?}"
    );
    assert_eq!(multi_team_recipient(&app, event).await, Some(team));
    assert_eq!(multi_team_balance(&app, member.id).await, 7);
    assert_eq!(multi_team_balance(&app, team).await, 11);
    assert_eq!(multi_team_land(&app, team).await, 3);
    let grants: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM ledger WHERE kind='achievement' AND detail->>'event_id'=$1",
    )
    .bind(event.to_string())
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(
        grants, 2,
        "The one accepted event grants each recipient exactly once"
    );
    let saved: Value = sqlx::query_scalar("SELECT payload FROM game_events WHERE id=$1")
        .bind(event)
        .fetch_one(&app.db)
        .await
        .unwrap();
    let accepted = multi_team_event(&member, session, event, saved["amount"].as_i64().unwrap());
    assert_eq!(
        multi_team_record(&app, server, accepted.clone()).await["duplicate"],
        true
    );
    let mut changed = accepted;
    changed["kind"] = json!("walk.distance");
    assert_eq!(
        internal(
            &app,
            "official",
            Some(server),
            "/internal/v1/game/event",
            changed
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(multi_team_balance(&app, team).await, 11);
}

#[sqlx::test(migrations = "../../migrations")]
async fn multi_team_failed_reward_rolls_back_all_recipients_and_resolves_on_successful_retry(
    pool: PgPool,
) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    let leader_a = account(&app, "Rollback A", false).await;
    let leader_b = account(&app, "Rollback B", false).await;
    let member = account(&app, "Rollback contributor", false).await;
    let (team_a, _) = multi_team_create(&app, &leader_a, "Rollback A").await;
    let (team_b, _) = multi_team_create(&app, &leader_b, "Rollback B").await;
    multi_team_join(&app, &leader_a, &member, team_a).await;
    multi_team_join(&app, &leader_b, &member, team_b).await;
    multi_team_rules(&app, 1).await;
    let session = multi_team_event_session(&app, &member, server).await;
    let event = Uuid::new_v4();
    let body = multi_team_event(&member, session, event, 1);
    let personal_land = multi_team_land(&app, member.id).await;
    // Simulate a storage failure after personal progress/reward writes, when the
    // subsequent selected-team grant is persisted. The public event must be atomic.
    sqlx::raw_sql(&format!("CREATE FUNCTION multi_team_abort_reward() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.owner='{team_a}'::uuid THEN RAISE EXCEPTION 'fixture reward persistence failure'; END IF; RETURN NEW; END; $$; CREATE TRIGGER multi_team_abort_reward BEFORE INSERT ON ledger_entries FOR EACH ROW EXECUTE FUNCTION multi_team_abort_reward();"))
        .execute(&app.db).await.unwrap();
    let (status, response) = internal(
        &app,
        "official",
        Some(server),
        "/internal/v1/game/event",
        body.clone(),
    )
    .await;
    assert!(status.is_server_error(), "{response}");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM game_events WHERE id=$1")
            .bind(event)
            .fetch_one(&app.db)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        multi_team_progress(&app, member.id, "multi_personal").await,
        0
    );
    assert_eq!(multi_team_progress(&app, team_a, "multi_shared").await, 0);
    assert_eq!(multi_team_balance(&app, member.id).await, 0);
    assert_eq!(multi_team_balance(&app, team_a).await, 0);
    assert_eq!(multi_team_land(&app, member.id).await, personal_land);
    assert_eq!(multi_team_land(&app, team_a).await, 0);
    sqlx::raw_sql("DROP TRIGGER multi_team_abort_reward ON ledger_entries; DROP FUNCTION multi_team_abort_reward();").execute(&app.db).await.unwrap();
    multi_team_ok(
        &app,
        &member,
        Command::TeamContributionSet { team: Some(team_b) },
    )
    .await;
    multi_team_record(&app, server, body.clone()).await;
    assert_eq!(
        multi_team_recipient(&app, event).await,
        Some(team_b),
        "A failed attempt does not freeze the contribution recipient"
    );
    assert_eq!(
        multi_team_record(&app, server, body).await["duplicate"],
        true
    );
    assert_eq!(multi_team_balance(&app, member.id).await, 7);
    assert_eq!(multi_team_balance(&app, team_b).await, 11);
    assert_eq!(multi_team_balance(&app, team_a).await, 0);
    assert_eq!(multi_team_land(&app, member.id).await, personal_land + 1);
    assert_eq!(multi_team_land(&app, team_b).await, 3);
}

#[sqlx::test(migrations = false)]
async fn multi_team_migration_preserves_existing_rights_progress_and_land(pool: PgPool) {
    for migration in sqlx::migrate!("../../migrations")
        .iter()
        .filter(|m| m.version < 20)
    {
        sqlx::raw_sql(&migration.sql).execute(&pool).await.unwrap();
    }
    let app = app(pool);
    let leader = account(&app, "Existing leader", false).await;
    let member = account(&app, "Existing member", false).await;
    let (server, _) = official(&app).await;
    let team = Uuid::new_v4();
    let room = Uuid::new_v4();
    let credential = Uuid::new_v4();
    let event = Uuid::new_v4();
    // Seed the historical one-team schema, before selection and recipient columns exist.
    sqlx::query("INSERT INTO principals(id,kind,name) VALUES($1,'team','Established team')")
        .bind(team)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO rooms(id,kind,name,owner) VALUES($1,'team','Established team',$2)")
        .bind(room)
        .bind(leader.id)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO teams(id,leader,room_id) VALUES($1,$2,$3)")
        .bind(team)
        .bind(leader.id)
        .bind(room)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members(team_id,account_id,can_build,can_sell,can_spend,can_manage_members,can_administer) VALUES($1,$2,true,true,true,true,true),($1,$3,false,true,false,true,false)")
        .bind(team).bind(leader.id).bind(member.id).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO room_members(room_id,account_id,role) VALUES($1,$2,'owner'),($1,$3,'moderator')")
        .bind(room).bind(leader.id).bind(member.id).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO land_allowances(owner,chunks) VALUES($1,37)")
        .bind(team)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO wallets(owner,balance) VALUES($1,250)")
        .bind(team)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO achievements(key,title,description,event,target,land_chunks,coins,team) VALUES('multi_existing','Existing goal','Existing goal','block.placed',200,3,11,true)").execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO achievement_progress(owner,achievement,progress) VALUES($1,'multi_existing',77)").bind(team).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO service_credentials(id,name,token_hash,role,server_id) VALUES($1,'Historical worker',$2,'official',$3)")
        .bind(credential).bind(auth::hash(&auth::random_token())).bind(server).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO game_events(id,credential,account_id,kind,payload) VALUES($1,$2,$3,'block.placed','{\"amount\":2}')")
        .bind(event).bind(credential).bind(member.id).execute(&app.db).await.unwrap();
    let snapshot_sql = "SELECT jsonb_build_object('membership',(SELECT to_jsonb(m) FROM team_members m WHERE team_id=$1 AND account_id=$2),'room',(SELECT to_jsonb(m) FROM room_members m WHERE room_id=$3 AND account_id=$2),'land',(SELECT to_jsonb(l) FROM land_allowances l WHERE owner=$1),'wallet',(SELECT to_jsonb(w) FROM wallets w WHERE owner=$1),'progress',(SELECT to_jsonb(p) FROM achievement_progress p WHERE owner=$1 AND achievement='multi_existing'),'event',(SELECT jsonb_build_object('id',id,'credential',credential,'account_id',account_id,'kind',kind,'payload',payload,'created_at',created_at) FROM game_events WHERE id=$4))";
    let before: Value = sqlx::query_scalar(snapshot_sql)
        .bind(team)
        .bind(member.id)
        .bind(room)
        .bind(event)
        .fetch_one(&app.db)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../../../../migrations/0020_multi_team.sql"))
        .execute(&app.db)
        .await
        .unwrap();
    let after: Value = sqlx::query_scalar(snapshot_sql)
        .bind(team)
        .bind(member.id)
        .bind(room)
        .bind(event)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(
        before, after,
        "The migration must preserve earned capacity, permissions, rooms, balances and historical events"
    );
    assert_eq!(
        sqlx::query_scalar::<_, Uuid>(
            "SELECT team_id FROM team_contribution_selection WHERE account_id=$1"
        )
        .bind(member.id)
        .fetch_one(&app.db)
        .await
        .unwrap(),
        team
    );
    assert_eq!(
        multi_team_recipient(&app, event).await,
        None,
        "Historical event recipient must not be invented during migration"
    );
    let new_leader = account(&app, "New leader", false).await;
    let (new_team, _) = multi_team_create(&app, &new_leader, "New team").await;
    assert_eq!(multi_team_land(&app, new_team).await, 0);
    multi_team_join(&app, &new_leader, &member, new_team).await;
    assert_eq!(multi_team_land(&app, team).await, 37);
    let detail = multi_team_detail(&app, &member, team).await;
    assert_eq!(detail["contribution_team_id"], json!(team));
    assert_eq!(detail["team"]["permissions"]["can_sell"], true);
    assert_eq!(detail["team"]["permissions"]["can_build"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn multi_team_account_link_unions_memberships_permissions_rooms_and_retains_selection(
    pool: PgPool,
) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    let retained = account(&app, "Retained account", false).await;
    let other = account(&app, "Linked account", false).await;
    let leader_a = account(&app, "Link A leader", false).await;
    let leader_b = account(&app, "Link B leader", false).await;
    let leader_shared = account(&app, "Shared leader", false).await;
    let (team_a, room_a) = multi_team_create(&app, &leader_a, "Retained membership").await;
    let (team_b, room_b) = multi_team_create(&app, &leader_b, "Linked membership").await;
    let (shared, shared_room) = multi_team_create(&app, &leader_shared, "Overlap").await;
    multi_team_join(&app, &leader_a, &retained, team_a).await;
    multi_team_join(&app, &leader_b, &other, team_b).await;
    multi_team_join(&app, &leader_shared, &retained, shared).await;
    multi_team_join(&app, &leader_shared, &other, shared).await;
    multi_team_permissions(
        &app,
        &leader_shared,
        &retained,
        shared,
        [true, false, true, false, false],
    )
    .await;
    multi_team_permissions(
        &app,
        &leader_shared,
        &other,
        shared,
        [false, true, false, true, false],
    )
    .await;
    sqlx::query("UPDATE room_members SET role='moderator' WHERE room_id=$1 AND account_id=$2")
        .bind(shared_room)
        .bind(other.id)
        .execute(&app.db)
        .await
        .unwrap();
    let begin = multi_team_ok(&app, &retained, Command::LinkBegin).await;
    multi_team_ok(
        &app,
        &other,
        Command::LinkPresent {
            code: begin["code"].as_str().unwrap().into(),
        },
    )
    .await;
    // Selecting the other's progression dataset must not replace the retained
    // account's independent contribution preference or drop social memberships.
    let selected: Uuid =
        sqlx::query_scalar("SELECT id FROM profiles WHERE account_id=$1 AND status='active'")
            .bind(other.id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    let migration = multi_team_ok(
        &app,
        &retained,
        Command::LinkConfirm {
            id: id(&begin, "id"),
            selected_profile: selected,
        },
    )
    .await;
    let payload: Value = sqlx::query_scalar("SELECT payload FROM jobs WHERE id=$1")
        .bind(id(&migration, "job_id"))
        .fetch_one(&app.db)
        .await
        .unwrap();
    // Native file migration has separate Paper acceptance. Here exercise its
    // authenticated completion receipt and the resulting social merge transaction.
    let proof = json!({"effect":"committed","native_data_verified":true,"native_uuid":payload["native_plan"]["canonical"],"archive_owner":payload["native_plan"]["archive_owner"],"manifest_sha256":"a".repeat(64),"pet_policy_durable":true});
    let (status, response) =
        acknowledge(&app, server, id(&migration, "job_id"), "succeeded", proof).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    let (status, social) = http(
        &app,
        &retained,
        "GET",
        "/api/v1/view/social?section=teams",
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{social}");
    let teams: std::collections::BTreeSet<Uuid> = social["teams"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| id(t, "id"))
        .collect();
    assert_eq!(teams, [team_a, team_b, shared].into_iter().collect());
    assert_eq!(social["contribution_team_id"], json!(team_a));
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM team_contribution_selection WHERE account_id=$1"
        )
        .bind(other.id)
        .fetch_one(&app.db)
        .await
        .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM team_members WHERE account_id=$1")
            .bind(other.id)
            .fetch_one(&app.db)
            .await
            .unwrap(),
        0
    );
    let detail = multi_team_detail(&app, &retained, shared).await;
    let member = detail["team"]["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["account_id"] == json!(retained.id))
        .unwrap();
    for flag in ["can_build", "can_sell", "can_spend", "can_manage_members"] {
        assert_eq!(
            member[flag], true,
            "{flag} should retain authority from either account"
        );
    }
    assert_eq!(
        member["can_administer"], false,
        "A merge must not invent a permission neither account held"
    );
    assert_eq!(
        detail["team"]["member_count"], 2,
        "The overlapping accounts become one membership"
    );
    for room in [room_a, room_b, shared_room] {
        multi_team_ok(
            &app,
            &retained,
            Command::MessageSend {
                room,
                body: "Linked room membership retained".into(),
            },
        )
        .await;
    }
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT role FROM room_members WHERE room_id=$1 AND account_id=$2"
        )
        .bind(shared_room)
        .bind(retained.id)
        .fetch_one(&app.db)
        .await
        .unwrap(),
        "moderator"
    );
}
