async fn teleport_fixture_player(app: &App, name: &str, server: Uuid) -> Actor {
    let actor = account(app, name, false).await;
    sqlx::query("INSERT INTO game_sessions(account_id,profile_id,native_uuid,session_id,server_id,lease_until) SELECT $1,id,$2,$3,$4,now()+interval '5 minutes' FROM profiles WHERE account_id=$1 AND status='active'")
        .bind(actor.id).bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(server).execute(&app.db).await.unwrap();
    actor
}

#[sqlx::test(migrations = "../../migrations")]
async fn teleport_directions_require_explicit_consent_and_preserve_job_privacy(pool: PgPool) {
    let app = app(pool);
    let (server,_) = official(&app).await;
    let sender = teleport_fixture_player(&app,"Requester",server).await;
    let recipient = teleport_fixture_player(&app,"Recipient",server).await;
    let outsider = teleport_fixture_player(&app,"Outsider",server).await;
    for here in [false,true] {
        let pending = run(&app,&sender,Command::TeleportRequest {target:recipient.id,here}).await;
        let invitation = id(&pending,"id");
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM jobs WHERE kind='player.teleport'").fetch_one(&app.db).await.unwrap(), if here {1} else {0});
        assert_eq!(multi_team_command(&app,&outsider,Command::InviteRespond {id:invitation,accept:true}).await.0,StatusCode::NOT_FOUND);
        let accepted = run(&app,&recipient,Command::InviteRespond {id:invitation,accept:true}).await;
        let job = id(&accepted,"job_id");
        let row = sqlx::query("SELECT actor,payload,state FROM jobs WHERE id=$1").bind(job).fetch_one(&app.db).await.unwrap();
        let payload:Value = row.get("payload");
        assert_eq!(row.get::<Uuid,_>("actor"),if here {recipient.id} else {sender.id});
        assert_eq!(payload["target"],json!(if here {sender.id} else {recipient.id}));
        assert_eq!(payload["accepted_by"],json!(recipient.id));
        assert_eq!(payload["invitation"],json!(invitation));
        assert_eq!(row.get::<String,_>("state"),"queued","Acceptance is not completed teleportation");
        for actor in [&sender,&recipient] {
            let (status,result) = http(&app,actor,"GET",&format!("/api/v1/jobs/{job}"),json!({}),false).await;
            assert_eq!(status,StatusCode::OK,"{result}");
            assert!(result.get("payload").is_none(),"Do not expose internal payloads with the outcome");
        }
        assert_eq!(http(&app,&outsider,"GET",&format!("/api/v1/jobs/{job}"),json!({}),false).await.0,StatusCode::NOT_FOUND);
        assert_eq!(multi_team_command(&app,&recipient,Command::InviteRespond {id:invitation,accept:true}).await.0,StatusCode::NOT_FOUND);
        let non_actor = if here {&sender} else {&recipient};
        sqlx::query("UPDATE jobs SET kind='home.travel' WHERE id=$1").bind(job).execute(&app.db).await.unwrap();
        assert_eq!(http(&app,non_actor,"GET",&format!("/api/v1/jobs/{job}"),json!({}),false).await.0,StatusCode::NOT_FOUND);
        sqlx::query("UPDATE jobs SET kind='player.teleport' WHERE id=$1").bind(job).execute(&app.db).await.unwrap();
        sqlx::query("INSERT INTO blocks(actor,target) VALUES($1,$2)").bind(recipient.id).bind(sender.id).execute(&app.db).await.unwrap();
        assert_eq!(http(&app,non_actor,"GET",&format!("/api/v1/jobs/{job}"),json!({}),false).await.0,StatusCode::NOT_FOUND);
        sqlx::query("DELETE FROM blocks WHERE actor=$1 AND target=$2").bind(recipient.id).bind(sender.id).execute(&app.db).await.unwrap();
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn teleport_pending_pairs_expire_replace_and_decline_without_movement(pool: PgPool) {
    let app = app(pool);
    let (server,_) = official(&app).await;
    let sender = teleport_fixture_player(&app,"Requester",server).await;
    let recipient = teleport_fixture_player(&app,"Recipient",server).await;
    let other = teleport_fixture_player(&app,"Other requester",server).await;
    let old = run(&app,&sender,Command::TeleportRequest {target:recipient.id,here:false}).await;
    let independent = run(&app,&other,Command::TeleportRequest {target:recipient.id,here:false}).await;
    let new = run(&app,&sender,Command::TeleportRequest {target:recipient.id,here:true}).await;
    assert_ne!(old["id"],new["id"]);
    assert_eq!(multi_team_command(&app,&recipient,Command::InviteRespond {id:id(&old,"id"),accept:true}).await.0,StatusCode::NOT_FOUND);
    assert_eq!(sqlx::query_scalar::<_,String>("SELECT state FROM invitations WHERE id=$1").bind(id(&independent,"id")).fetch_one(&app.db).await.unwrap(),"pending");
    run(&app,&recipient,Command::InviteRespond {id:id(&new,"id"),accept:false}).await;
    sqlx::query("UPDATE invitations SET expires_at=now()-interval '1 second' WHERE id=$1").bind(id(&independent,"id")).execute(&app.db).await.unwrap();
    assert_eq!(multi_team_command(&app,&recipient,Command::InviteRespond {id:id(&independent,"id"),accept:true}).await.0,StatusCode::NOT_FOUND);
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM jobs WHERE kind='player.teleport'").fetch_one(&app.db).await.unwrap(),0);
    let offline = run(&app,&sender,Command::TeleportRequest {target:recipient.id,here:false}).await;
    sqlx::query("UPDATE game_sessions SET lease_until=now()-interval '1 second' WHERE account_id=$1").bind(sender.id).execute(&app.db).await.unwrap();
    assert!(multi_team_command(&app,&recipient,Command::InviteRespond {id:id(&offline,"id"),accept:true}).await.0.is_client_error());
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM jobs WHERE kind='player.teleport'").fetch_one(&app.db).await.unwrap(),0);
}

#[test]
fn optional_action_context_preserves_legacy_serialized_command_identity() {
    let target = Uuid::new_v4();
    for value in [json!({"type":"party_ready","ready":true}),json!({"type":"party_leave"}),json!({"type":"party_transfer","target":target}),json!({"type":"teleport_request","target":target})] {
        let command: Command = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(command).unwrap(),value,"Adding an optional context must not change a legacy request's idempotency hash");
    }
}
