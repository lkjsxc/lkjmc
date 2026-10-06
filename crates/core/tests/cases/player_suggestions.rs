async fn picker_session(app: &App, actor: &Actor, server: Uuid) {
    let native = Uuid::new_v4();
    sqlx::query("UPDATE profiles SET native_uuid=$2 WHERE account_id=$1 AND status='active'")
        .bind(actor.id).bind(native).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO game_sessions(account_id,profile_id,native_uuid,session_id,server_id,lease_until) SELECT $1,id,$2,$3,$4,now()+interval '5 minutes' FROM profiles WHERE account_id=$1 AND status='active'")
        .bind(actor.id).bind(native).bind(Uuid::new_v4()).bind(server).execute(&app.db).await.unwrap();
}
async fn picker_rows(app: &App, actor: &Actor, path: &str) -> Vec<Value> {
    let (status, value) = http(app, actor, "GET", path, json!({}), false).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value["players"].as_array().unwrap().clone()
}
#[sqlx::test(migrations = "../../migrations")]
async fn player_suggestions_prioritize_shared_context_without_listing_strangers(pool: PgPool) {
    let app = app(pool);
    let viewer = account(&app, "Viewer", false).await;
    let nearby = account(&app, "Z Nearby", false).await;
    let party_member = account(&app, "Y Party", false).await;
    let teammate = account(&app, "X Team", false).await;
    let friend = account(&app, "W Friend", false).await;
    let pending = account(&app, "A Pending", false).await;
    let _stranger = account(&app, "A Stranger", false).await;
    let (server, _) = official(&app).await;
    for who in [&viewer, &nearby] { picker_session(&app, who, server).await; }
    let party = run(&app, &viewer, Command::PartyCreate { name: "Fixture".into() }).await;
    let invitation = run(&app, &viewer, Command::Invite { kind:"party".into(),resource:id(&party,"party_id"),target:party_member.id }).await;
    run(&app, &party_member, Command::InviteRespond { id:id(&invitation,"id"),accept:true }).await;
    let (team, _) = multi_team_create(&app, &viewer, "Team").await;
    multi_team_join(&app, &viewer, &teammate, team).await;
    for who in [&friend, &pending] { run(&app, &viewer, Command::FriendRequest { target: who.id }).await; }
    run(&app, &friend, Command::FriendRespond {target:viewer.id,accept:true}).await;
    let rows = picker_rows(&app,&viewer,"/api/v1/players").await;
    assert_eq!(rows.iter().map(|v|v["id"].clone()).collect::<Vec<_>>(),vec![json!(nearby.id),json!(party_member.id),json!(teammate.id),json!(friend.id)]);
    for row in &rows {
        let keys = row.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>();
        assert_eq!(keys, vec!["id","name","rank","rank_message"]);
    }
    run(&app, &viewer, Command::Block { target:nearby.id,blocked:true }).await;
    run(&app, &teammate, Command::Block { target:viewer.id,blocked:true }).await;
    let rows = picker_rows(&app,&viewer,"/api/v1/players?q=").await;
    assert_eq!(rows.iter().map(|v|v["id"].clone()).collect::<Vec<_>>(),vec![json!(party_member.id),json!(friend.id)]);
    run(&app,&party_member,Command::PartyLeave { party: None }).await;
    assert_eq!(picker_rows(&app,&viewer,"/api/v1/players").await.len(),1);
}
#[sqlx::test(migrations = "../../migrations")]
async fn player_suggestions_revalidate_private_access_hidden_presence_and_session_identity(pool: PgPool) {
    let app = app(pool);
    let viewer = account(&app,"Viewer",false).await;
    let nearby = account(&app,"Nearby",false).await;
    let (server, _) = official(&app).await;
    for who in [&viewer,&nearby] { picker_session(&app,who,server).await; }
    assert_eq!(picker_rows(&app,&viewer,"/api/v1/players").await.len(),1);
    sqlx::query("UPDATE servers SET visibility='private' WHERE id=$1").bind(server).execute(&app.db).await.unwrap();
    assert!(picker_rows(&app,&viewer,"/api/v1/players").await.is_empty());
    sqlx::query("INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,'guest')").bind(server).bind(viewer.id).execute(&app.db).await.unwrap();
    assert_eq!(picker_rows(&app,&viewer,"/api/v1/players").await.len(),1);
    sqlx::query("UPDATE accounts SET activity_policy='none' WHERE id=$1").bind(nearby.id).execute(&app.db).await.unwrap();
    assert!(picker_rows(&app,&viewer,"/api/v1/players").await.is_empty());
    let (team,_) = multi_team_create(&app,&viewer,"Known membership").await;
    multi_team_join(&app,&viewer,&nearby,team).await;
    let known = picker_rows(&app,&viewer,"/api/v1/players").await;
    assert_eq!(known.len(),1,"Membership identifies a known player without revealing hidden activity");
    assert!(known[0].get("server_id").is_none());
    run(&app,&nearby,Command::TeamLeave{team}).await;
    sqlx::query("UPDATE accounts SET activity_policy='friends' WHERE id=$1").bind(nearby.id).execute(&app.db).await.unwrap();
    sqlx::query("UPDATE game_sessions SET lease_until=now()-interval '1 second' WHERE account_id=$1").bind(nearby.id).execute(&app.db).await.unwrap();
    assert!(picker_rows(&app,&viewer,"/api/v1/players").await.is_empty());
    sqlx::query("UPDATE game_sessions SET lease_until=now()+interval '5 minutes',native_uuid=$2 WHERE account_id=$1").bind(nearby.id).bind(Uuid::new_v4()).execute(&app.db).await.unwrap();
    assert!(picker_rows(&app,&viewer,"/api/v1/players").await.is_empty());
}
#[sqlx::test(migrations = "../../migrations")]
async fn player_name_search_stays_literal_bounded_and_cannot_bypass_blocks(pool: PgPool) {
    let app = app(pool);
    let viewer = account(&app,"Viewer",true).await;
    let literal = account(&app,"Literal %_ player",false).await;
    for i in 0..35 { account(&app,&format!("Member {i:02}"),false).await; }
    assert!(picker_rows(&app,&viewer,"/api/v1/players").await.is_empty());
    let rows = picker_rows(&app,&viewer,"/api/v1/players?q=%25_").await;
    assert_eq!(rows.len(),1); assert_eq!(rows[0]["id"],json!(literal.id));
    let rows = picker_rows(&app,&viewer,"/api/v1/players?q=Member").await;
    assert_eq!(rows.len(),30);
    assert_eq!(rows[0]["name"],"Member 00"); assert_eq!(rows[29]["name"],"Member 29");
    assert!(picker_rows(&app,&viewer,&format!("/api/v1/players?q={}","x".repeat(129))).await.is_empty());
    run(&app,&literal,Command::Block{target:viewer.id,blocked:true}).await;
    assert!(picker_rows(&app,&viewer,"/api/v1/players?q=Literal").await.is_empty());
    assert!(picker_rows(&app,&viewer,&format!("/api/v1/players?q={}",literal.id)).await.is_empty());
}
