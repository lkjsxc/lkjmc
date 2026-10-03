#[sqlx::test(migrations = "../../migrations")]
async fn passive_reads_do_not_create_completed_action_notifications(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Owner", false).await;
    let server = custom_server(&app, owner.id).await;
    let token = host_token(&app).await;
    let credential: Uuid = sqlx::query_scalar("SELECT id FROM service_credentials WHERE token_hash=$1")
        .bind(auth::hash(&token)).fetch_one(&app.db).await.unwrap();
    let mut explicit_job = None;
    for kind in ["server.logs", "server.files", "server.file.read", "server.console"] {
        let job = Uuid::new_v4();
        let lease = Uuid::new_v4();
        let payload = match kind {
            "server.files" => json!({"path":""}),
            "server.file.read" => json!({"path":"readme.txt"}),
            "server.console" => json!({"command":"help"}),
            _ => json!({}),
        };
        sqlx::query("INSERT INTO jobs(id,actor,server_id,worker,kind,payload,state,lease_owner,lease_token,lease_until,host_authorized_at) VALUES($1,$2,$3,'host',$4,$5,'leased',$6,$7,now()+interval '90 seconds',now())")
            .bind(job).bind(owner.id).bind(server).bind(kind).bind(payload).bind(credential).bind(lease)
            .execute(&app.db).await.unwrap();
        let result = match kind {
            "server.logs" => json!({"lines":[]}),
            "server.files" => json!({"path":"","entries":[]}),
            "server.file.read" => json!({"path":"readme.txt","text":"","sha256":auth::hash(""),"bytes":0}),
            _ => json!({"effect":"committed"}),
        };
        let body = json!({"lease_token":lease,"state":"succeeded","result":result});
        let path = format!("/internal/v1/jobs/{job}/ack");
        let (status, response) = host_http(&app, &token, &path, body.clone()).await;
        assert_eq!(status, StatusCode::OK, "{kind}: {response}");
        assert_eq!(host_http(&app, &token, &path, body).await.0, StatusCode::OK);
        if kind == "server.console" { explicit_job = Some(job); }
    }
    let notices: Vec<Value> = sqlx::query_scalar("SELECT body FROM notifications WHERE account_id=$1")
        .bind(owner.id).fetch_all(&app.db).await.unwrap();
    assert_eq!(notices.len(), 1, "passive refresh must not fill notifications");
    assert_eq!(notices[0]["id"], explicit_job.unwrap().to_string());
    assert_eq!(notices[0]["kind"], "server.console");
    assert_eq!(notices[0]["state"], "succeeded");
    assert_eq!(notices[0]["server_id"], server.to_string());
}
