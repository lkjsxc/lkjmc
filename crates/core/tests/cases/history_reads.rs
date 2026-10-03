#[sqlx::test(migrations = "../../migrations")]
async fn notification_history_does_not_surface_passive_reads_or_their_old_results(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Owner", true).await;
    let other = account(&app, "Other", false).await;
    for actor in [owner.id, other.id] {
        for kind in ["server.logs", "server.files", "server.file.read", "server.start"] {
            let id = Uuid::new_v4();
            let passive = kind != "server.start";
            sqlx::query("INSERT INTO jobs(id,actor,worker,kind,payload,state,result) VALUES($1,$2,'host',$3,'{}','waiting',$4)")
                .bind(id).bind(actor).bind(kind)
                .bind(if passive {json!({"text":"private read result must stay out of activity"})} else {json!({"state":"starting"})})
                .execute(&app.db).await.unwrap();
            sqlx::query("INSERT INTO notifications(account_id,kind,body) VALUES($1,'job_finished',$2)")
                .bind(actor).bind(json!({"id":id,"kind":kind,"state":"succeeded"}))
                .execute(&app.db).await.unwrap();
        }
        sqlx::query("INSERT INTO notifications(account_id,kind,body) VALUES($1,'achievement','{}')")
            .bind(actor).execute(&app.db).await.unwrap();
    }
    for path in ["/api/v1/home", "/api/v1/view/home", "/api/v1/view/home?section=overview"] {
        let (status, view) = http(&app,&owner,"GET",path,json!({}),false).await;
        assert_eq!(status,StatusCode::OK,"{path}: {view}");
        assert_eq!(view["jobs"].as_array().unwrap().len(),1,"{path}: {view}");
        assert_eq!(view["notifications"].as_array().unwrap().len(),2,"{path}: {view}");
        assert!(!view.to_string().contains("private read result"));
        if path=="/api/v1/home" {
            assert_eq!(view["counts"]["jobs"],1);
            assert_eq!(view["counts"]["notifications"],2);
        }
    }
    let (_, jobs) = http(&app,&owner,"GET","/api/v1/history/activity",json!({}),false).await;
    assert_eq!(jobs["jobs"].as_array().unwrap().len(),1,"{jobs}");
    assert!(!jobs.to_string().contains("private read result"));
    let (_, notices) = http(&app,&owner,"GET","/api/v1/history/notifications",json!({}),false).await;
    assert_eq!(notices["notifications"].as_array().unwrap().len(),2,"{notices}");
    let (_, admin) = http(&app,&owner,"GET","/api/v1/view/admin?section=jobs",json!({}),false).await;
    assert_eq!(admin["jobs"].as_array().unwrap().len(),2,"{admin}");
    let (_, admin) = http(&app,&owner,"GET","/api/v1/view/admin?section=overview",json!({}),false).await;
    assert_eq!(admin["counts"]["jobs"],2,"{admin}");
}
