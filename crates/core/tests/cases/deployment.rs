#[sqlx::test(migrations = "../../migrations")]
async fn deployment_gate_fences_inflight_transactions_and_new_admission(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "owner", true).await;
    let mut operation = app.db.begin().await.unwrap();
    lkjmc_core::deployment::enter(&mut operation).await.unwrap();
    let mut manager = app.db.begin().await.unwrap();
    let locked: bool = sqlx::query_scalar(
        "SELECT pg_try_advisory_xact_lock(hashtextextended('lkjmc-deployment',0))",
    ).fetch_one(&mut *manager).await.unwrap();
    assert!(!locked, "manager cannot overtake an in-flight operation");
    operation.commit().await.unwrap();
    let locked: bool = sqlx::query_scalar(
        "SELECT pg_try_advisory_xact_lock(hashtextextended('lkjmc-deployment',0))",
    ).fetch_one(&mut *manager).await.unwrap();
    assert!(locked);
    sqlx::query("INSERT INTO settings(key,value) VALUES('deployment_gate',$1)")
        .bind(json!({"closed":true,"owner":"fixture"}))
        .execute(&mut *manager).await.unwrap();
    manager.commit().await.unwrap();
    let mutation = commands::execute(&app, &owner, Request {
        request_id: Uuid::new_v4(),
        command: Command::RankSet { target: owner.id, rank: 0 },
    }).await.unwrap_err();
    assert_eq!(mutation.status, StatusCode::SERVICE_UNAVAILABLE);
    let player = Uuid::new_v4();
    let connect = json!({"issuer":"java","subject":player.to_string(),
        "native_uuid":player,"session_id":Uuid::new_v4(),"display_name":"NewPlayer"});
    let (status, _) = internal(&app, "proxy", None, "/internal/v1/game/connect", connect.clone()).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let identities: i64 = sqlx::query_scalar("SELECT count(*) FROM identities WHERE issuer='java'")
        .fetch_one(&app.db).await.unwrap();
    assert_eq!(identities, 0, "rejected admission must not create a player");
    let (status, _) = http(&app, &owner, "GET", "/api/v1/view/home", json!({}), true).await;
    assert_eq!(status, StatusCode::OK, "read-only Web remains available");
    sqlx::query("UPDATE settings SET value=$1 WHERE key='deployment_gate'")
        .bind(json!({"closed":false,"owner":"fixture"})).execute(&app.db).await.unwrap();
    let (status, _) = internal(&app, "proxy", None, "/internal/v1/game/connect", connect).await;
    assert_eq!(status, StatusCode::OK);
    run(&app, &owner, Command::RankSet { target: owner.id, rank: 0 }).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn deployment_cli_waits_for_operations_and_activity_and_fences_reopening(pool: PgPool) {
    use lkjmc_core::deployment::{Action, control};
    let app = app(pool);
    let owner = Uuid::new_v4();
    let other = Uuid::new_v4();
    let mut tx = app.db.begin().await.unwrap();
    lkjmc_core::deployment::enter(&mut tx).await.unwrap();
    assert_eq!(control(&app.db, Action::Close, Some(owner)).await.unwrap()["status"], "waiting-for-operations");
    tx.commit().await.unwrap();
    let closed = control(&app.db, Action::Close, Some(owner)).await.unwrap();
    assert_eq!(closed["status"], "closed");
    assert_eq!(closed["gate"]["owner"], json!(owner));
    assert_eq!(control(&app.db, Action::Close, Some(owner)).await.unwrap(), closed);
    assert!(control(&app.db, Action::Close, Some(other)).await.is_err());
    assert!(control(&app.db, Action::Open, Some(other)).await.is_err());
    assert_eq!(control(&app.db, Action::Open, Some(owner)).await.unwrap()["status"], "open");
    let player = Uuid::new_v4();
    let (status, _) = internal(&app, "proxy", None, "/internal/v1/game/connect", json!({
        "issuer":"java","subject":player.to_string(),"native_uuid":player,
        "session_id":Uuid::new_v4(),"display_name":"Playing"
    })).await;
    assert_eq!(status, StatusCode::OK);
    let waiting = control(&app.db, Action::Close, Some(other)).await.unwrap();
    assert_eq!(waiting["status"], "waiting-for-idle");
    assert_eq!(waiting["activity"]["sessions"], 1);
    assert_eq!(waiting["gate"]["closed"], false);
}
