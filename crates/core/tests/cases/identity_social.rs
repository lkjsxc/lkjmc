#[sqlx::test(migrations = "../../migrations")]
async fn identity_merge_retains_social_authority_invitations_and_late_restrictions(pool: PgPool) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    let retained = account(&app, "連携元", false).await;
    let other = account(&app, "連携先", false).await;
    let friend = account(&app, "共通の友人", false).await;
    let blocked = account(&app, "ブロック対象", false).await;
    let admin = account(&app, "管理者", true).await;
    run(
        &app,
        &retained,
        Command::FriendRequest { target: friend.id },
    )
    .await;
    run(&app, &other, Command::FriendRequest { target: friend.id }).await;
    run(
        &app,
        &friend,
        Command::FriendRespond {
            target: other.id,
            accept: true,
        },
    )
    .await;
    run(
        &app,
        &retained,
        Command::FriendRequest { target: blocked.id },
    )
    .await;
    run(
        &app,
        &blocked,
        Command::FriendRespond {
            target: retained.id,
            accept: true,
        },
    )
    .await;
    run(
        &app,
        &other,
        Command::Block {
            target: blocked.id,
            blocked: true,
        },
    )
    .await;
    let own_room = id(
        &run(
            &app,
            &other,
            Command::RoomCreate {
                name: "連携先が所有する会話".into(),
            },
        )
        .await,
        "room_id",
    );
    let invite = run(
        &app,
        &other,
        Command::Invite {
            kind: "room".into(),
            resource: own_room,
            target: retained.id,
        },
    )
    .await;
    run(
        &app,
        &retained,
        Command::InviteRespond {
            id: id(&invite, "id"),
            accept: true,
        },
    )
    .await;
    let outgoing = run(
        &app,
        &other,
        Command::Invite {
            kind: "room".into(),
            resource: own_room,
            target: friend.id,
        },
    )
    .await;
    let friend_room = id(
        &run(
            &app,
            &friend,
            Command::RoomCreate {
                name: "友人の会話".into(),
            },
        )
        .await,
        "room_id",
    );
    let kept = run(
        &app,
        &friend,
        Command::Invite {
            kind: "room".into(),
            resource: friend_room,
            target: retained.id,
        },
    )
    .await;
    let duplicate = run(
        &app,
        &friend,
        Command::Invite {
            kind: "room".into(),
            resource: friend_room,
            target: other.id,
        },
    )
    .await;
    let distinct_room = id(
        &run(
            &app,
            &friend,
            Command::RoomCreate {
                name: "別の会話".into(),
            },
        )
        .await,
        "room_id",
    );
    let moved = run(
        &app,
        &friend,
        Command::Invite {
            kind: "room".into(),
            resource: distinct_room,
            target: other.id,
        },
    )
    .await;
    let blocked_room = id(
        &run(
            &app,
            &blocked,
            Command::RoomCreate {
                name: "遮断する招待".into(),
            },
        )
        .await,
        "room_id",
    );
    let cancelled = run(
        &app,
        &blocked,
        Command::Invite {
            kind: "room".into(),
            resource: blocked_room,
            target: retained.id,
        },
    )
    .await;
    let team = id(
        &run(
            &app,
            &friend,
            Command::TeamCreate {
                name: "共通チーム".into(),
            },
        )
        .await,
        "team_id",
    );
    for actor in [&retained, &other] {
        let invitation = run(
            &app,
            &friend,
            Command::Invite {
                kind: "team".into(),
                resource: team,
                target: actor.id,
            },
        )
        .await;
        run(
            &app,
            actor,
            Command::InviteRespond {
                id: id(&invitation, "id"),
                accept: true,
            },
        )
        .await;
    }
    sqlx::query("UPDATE team_members SET can_sell=true WHERE account_id=$1")
        .bind(other.id)
        .execute(&app.db)
        .await
        .unwrap();
    let report = id(
        &run(
            &app,
            &other,
            Command::Report {
                target: Some(friend.id),
                reason: "本人が提出した内容".into(),
                message_ids: vec![],
            },
        )
        .await,
        "report_id",
    );
    let evidence: Value = sqlx::query_scalar("SELECT evidence FROM reports WHERE id=$1")
        .bind(report)
        .fetch_one(&app.db)
        .await
        .unwrap();
    let begin = run(&app, &retained, Command::LinkBegin).await;
    run(
        &app,
        &other,
        Command::LinkPresent {
            code: begin["code"].as_str().unwrap().into(),
        },
    )
    .await;
    let profile: Uuid = sqlx::query_scalar("SELECT id FROM profiles WHERE account_id=$1")
        .bind(retained.id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    let migration = run(
        &app,
        &retained,
        Command::LinkConfirm {
            id: id(&begin, "id"),
            selected_profile: profile,
        },
    )
    .await;
    assert!(
        commands::execute(
            &app,
            &friend,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::Invite {
                    kind: "room".into(),
                    resource: distinct_room,
                    target: other.id
                }
            }
        )
        .await
        .is_err(),
        "Do not send new invitations into a moving profile"
    );
    // A moderator acts after migration starts. Completing physical recovery must
    // retain this restriction rather than attach the identity to an unbanned account.
    run(
        &app,
        &admin,
        Command::Ban {
            target: other.id,
            hours: 24,
            reason: "移行中の利用制限".into(),
        },
    )
    .await;
    let payload: Value = sqlx::query_scalar("SELECT payload FROM jobs WHERE id=$1")
        .bind(id(&migration, "job_id"))
        .fetch_one(&app.db)
        .await
        .unwrap();
    // Core settlement protocol fixture; native file migration is tested on Paper separately.
    let proof = json!({"effect":"committed","native_data_verified":true,"native_uuid":payload["native_plan"]["canonical"],"archive_owner":payload["native_plan"]["archive_owner"],"manifest_sha256":"a".repeat(64),"pet_policy_durable":true});
    let (status, response) =
        acknowledge(&app, server, id(&migration, "job_id"), "succeeded", proof).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    assert!(
        sqlx::query_scalar::<_, bool>(
            "SELECT banned_until>now()+interval '23 hours' FROM accounts WHERE id=$1"
        )
        .bind(retained.id)
        .fetch_one(&app.db)
        .await
        .unwrap()
    );
    run(
        &app,
        &admin,
        Command::Ban {
            target: retained.id,
            hours: 0,
            reason: "制限継承の試験完了".into(),
        },
    )
    .await;
    let mut db = app.db.acquire().await.unwrap();
    assert_eq!(
        lkjmc_core::social::room_member(&mut db, own_room, retained.id)
            .await
            .unwrap(),
        "owner"
    );
    assert!(
        lkjmc_core::social::friends(&mut db, retained.id, friend.id)
            .await
            .unwrap()
    );
    assert!(
        !lkjmc_core::social::friends(&mut db, retained.id, blocked.id)
            .await
            .unwrap()
    );
    assert!(
        lkjmc_core::social::unblocked(&mut db, retained.id, blocked.id)
            .await
            .is_err()
    );
    assert!(
        lkjmc_core::social::unblocked(&mut db, friend.id, other.id)
            .await
            .is_err()
    );
    drop(db);
    assert!(
        sqlx::query_scalar::<_, bool>(
            "SELECT can_sell FROM team_members WHERE account_id=$1 AND team_id=$2"
        )
        .bind(retained.id)
        .bind(team)
        .fetch_one(&app.db)
        .await
        .unwrap()
    );
    for invitation in [&duplicate, &cancelled] {
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT state FROM invitations WHERE id=$1")
                .bind(id(invitation, "id"))
                .fetch_one(&app.db)
                .await
                .unwrap(),
            "cancelled"
        );
    }
    for invitation in [&kept, &moved] {
        run(
            &app,
            &retained,
            Command::InviteRespond {
                id: id(invitation, "id"),
                accept: true,
            },
        )
        .await;
    }
    run(
        &app,
        &friend,
        Command::InviteRespond {
            id: id(&outgoing, "id"),
            accept: true,
        },
    )
    .await;
    let (status, report_data) = http(
        &app,
        &retained,
        "GET",
        &format!("/api/v1/reports/{report}"),
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{report_data}");
    assert_eq!(report_data["evidence"], evidence);
}
