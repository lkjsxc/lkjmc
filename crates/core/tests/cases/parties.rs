#[sqlx::test(migrations = "../../migrations")]
async fn unnamed_party_creation_is_singular_and_keeps_existing_membership(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Party creator", false).await;
    let command: Command = serde_json::from_value(json!({"type":"party_create"})).unwrap();
    let (a, b) = tokio::join!(run(&app, &owner, command.clone()), run(&app, &owner, command));
    assert_eq!(a["party_id"], b["party_id"]);
    assert_eq!(a["room_id"], b["room_id"]);
    let room = id(&a, "room_id");
    let saved: String = sqlx::query_scalar("SELECT name FROM rooms WHERE id=$1")
        .bind(room).fetch_one(&app.db).await.unwrap();
    assert_eq!(saved, "Party");
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM rooms WHERE owner=$1 AND kind='party'")
        .bind(owner.id).fetch_one(&app.db).await.unwrap();
    assert_eq!(count, 1, "Duplicate clicks must not leave orphan rooms");
    let repeat = run(&app, &owner, Command::PartyCreate { name: "Do not replace this name".into() }).await;
    assert_eq!(repeat["party_id"], a["party_id"]);
    assert_eq!(sqlx::query_scalar::<_,String>("SELECT name FROM rooms WHERE id=$1").bind(room).fetch_one(&app.db).await.unwrap(), saved);
    run(&app, &owner, Command::PartyLeave).await;
    run(&app, &owner, Command::Language { language: "ja".into() }).await;
    let next = run(&app, &owner, Command::PartyCreate { name: String::new() }).await;
    assert_ne!(next["party_id"], a["party_id"]);
    assert_eq!(sqlx::query_scalar::<_,String>("SELECT name FROM rooms WHERE id=$1").bind(id(&next,"room_id")).fetch_one(&app.db).await.unwrap(), "パーティー");
}

#[sqlx::test(migrations = "../../migrations")]
async fn party_rename_requires_current_leader_and_preserves_private_data(pool: PgPool) {
    let app = app(pool);
    let leader = account(&app, "Party leader", false).await;
    let member = account(&app, "Party member", false).await;
    let outsider = account(&app, "Site administrator", true).await;
    let created = run(&app, &leader, Command::PartyCreate { name: "Original".into() }).await;
    let party = id(&created, "party_id");
    let room = id(&created, "room_id");
    let invitation = run(&app, &leader, Command::Invite { kind: "party".into(), resource: party, target: member.id }).await;
    run(&app, &member, Command::InviteRespond { id: id(&invitation,"id"), accept:true }).await;
    run(&app, &leader, Command::MessageSend { room, body: "Preserve our message".into() }).await;
    for actor in [&member, &outsider] {
        let (status, _) = multi_team_command(&app, actor, Command::PartyRename { party, name: "Not permitted".into() }).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    let renamed = run(&app, &leader, Command::PartyRename { party, name: "  次の冒険  ".into() }).await;
    assert_eq!(renamed["party_id"], json!(party));
    assert_eq!(renamed["name"], "次の冒険");
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM party_members WHERE party_id=$1").bind(party).fetch_one(&app.db).await.unwrap(), 2);
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM messages WHERE room_id=$1 AND body='Preserve our message'").bind(room).fetch_one(&app.db).await.unwrap(), 1);
    run(&app, &leader, Command::PartyTransfer { target: member.id }).await;
    assert_eq!(multi_team_command(&app, &leader, Command::PartyRename { party, name: "Old leader".into() }).await.0, StatusCode::FORBIDDEN);
    run(&app, &member, Command::PartyRename { party, name: "Current leader".into() }).await;
    for name in [String::new(), "a".repeat(81)] {
        assert!(multi_team_command(&app, &member, Command::PartyRename { party, name }).await.0.is_client_error());
    }
    assert_eq!(sqlx::query_scalar::<_,String>("SELECT name FROM rooms WHERE id=$1").bind(room).fetch_one(&app.db).await.unwrap(), "Current leader");
}

#[sqlx::test(migrations = "../../migrations")]
async fn stale_party_rename_cannot_mutate_a_new_membership(pool: PgPool) {
    let app = app(pool);
    let leader = account(&app, "Changing party leader", false).await;
    let old = run(&app, &leader, Command::PartyCreate { name: "Original".into() }).await;
    let old_party = id(&old, "party_id");
    run(&app, &leader, Command::PartyLeave).await;
    let new = run(&app, &leader, Command::PartyCreate { name: "New party".into() }).await;
    let stale = Command::PartyRename { party: old_party, name: "Stale rename".into() };
    assert_eq!(multi_team_command(&app, &leader, stale).await.0, StatusCode::FORBIDDEN);
    for (room, expected) in [(id(&old, "room_id"), "Original"), (id(&new, "room_id"), "New party")] {
        let actual: String = sqlx::query_scalar("SELECT name FROM rooms WHERE id=$1")
            .bind(room).fetch_one(&app.db).await.unwrap();
        assert_eq!(actual, expected);
    }
    assert!(serde_json::from_value::<Command>(json!({"type":"party_rename","name":"No target"})).is_err());
}
