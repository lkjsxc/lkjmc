// Included by integration.rs so these scenarios share the real PostgreSQL/API fixtures.
#[sqlx::test(migrations = "../../migrations")]
async fn backup_schedule_is_durable_singleton_and_catches_up_only_the_latest_slot(pool: PgPool) {
    use chrono::{Duration, TimeZone, Utc};
    let mut app = app(pool);
    let (server, _) = official(&app).await;
    let admin = account(&app, "自動保存の設定者", true).await;
    let now = Utc.with_ymd_and_hms(2026, 10, 1, 20, 0, 0).unwrap();
    lkjmc_core::services::backup_maintenance(&app, now)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM backups")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        0,
        "Development must not schedule host work"
    );
    Arc::make_mut(&mut app.config).development = false;
    let (a, b) = tokio::join!(
        lkjmc_core::services::backup_maintenance(&app, now),
        lkjmc_core::services::backup_maintenance(&app, now)
    );
    a.unwrap();
    b.unwrap();
    let backup: Value = sqlx::query_scalar("SELECT to_jsonb(b) FROM backups b")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(backup["scheduled_for"], "2026-10-01");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM backups")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        1
    );
    // A change in the human administrator does not orphan scheduled system work.
    sqlx::query("UPDATE accounts SET administrator=false WHERE id=$1")
        .bind(admin.id)
        .execute(&app.db)
        .await
        .unwrap();
    let token = host_token(&app).await;
    let (_, response) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    let job = response["job"].clone();
    let route = format!("/internal/v1/jobs/{}/context", job["id"].as_str().unwrap());
    let (status, context) = host_http(
        &app,
        &token,
        &route,
        json!({"lease_token":job["lease_token"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{context}");
    assert!(context["rejected"].is_null(), "{context}");
    // Scheduling fixture: completion is simulated here, not an actual VM backup.
    sqlx::query("UPDATE backups SET state='ready',completed_at=$1,manifest='{}'")
        .bind(now)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("UPDATE jobs SET state='succeeded'")
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("UPDATE servers SET maintenance=false,maintenance_job_id=NULL WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET administrator=true WHERE id=$1")
        .bind(admin.id)
        .execute(&app.db)
        .await
        .unwrap();
    lkjmc_core::services::backup_maintenance(&app, now)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM backups")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        1
    );
    let later = now + Duration::days(14);
    lkjmc_core::services::backup_maintenance(&app, later)
        .await
        .unwrap();
    let dates: Vec<String> =
        sqlx::query_scalar("SELECT scheduled_for::text FROM backups ORDER BY scheduled_for")
            .fetch_all(&app.db)
            .await
            .unwrap();
    assert_eq!(
        dates,
        vec!["2026-10-01", "2026-10-15"],
        "Do not backfill 13 nonexistent historical snapshots"
    );
    lkjmc_core::services::backup_maintenance(&app, later + Duration::days(1))
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM backups")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        2,
        "An unfinished backup prevents another stop/freeze"
    );
}

async fn backup_generation(
    app: &App,
    actor: Uuid,
    server: Uuid,
    when: chrono::DateTime<chrono::Utc>,
    state: &str,
    scheduled: bool,
) -> Uuid {
    use sha2::{Digest, Sha256};
    let backup = Uuid::new_v4();
    let job = Uuid::new_v4();
    let bytes = b"retention database fixture";
    let manifest = json!({"verified":true,"fixture":true,"backup_id":backup,"database":{"sha256":hex::encode(Sha256::digest(bytes)),"bytes":bytes.len()},"world":{"server_id":server,"sha256":"0".repeat(64),"bytes":1}});
    sqlx::query("INSERT INTO jobs(id,actor,server_id,worker,kind,payload,state,created_at,updated_at) VALUES($1,$2,$3,'host','official.backup',$4,$5,$6,$6)")
        .bind(job).bind(actor).bind(server).bind(json!({"backup_id":backup})).bind(if state=="failed"{"failed"}else{"succeeded"}).bind(when).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO backups(id,server_id,kind,state,manifest,created_at,completed_at,scheduled_for,job_id) VALUES($1,$2,'official',$3,$4,$5,$6,$7,$8)")
        .bind(backup).bind(server).bind(state).bind(manifest).bind(when).bind(if state=="ready"{Some(when)}else{None})
        .bind(if scheduled{Some(when.date_naive())}else{None}).bind(job).execute(&app.db).await.unwrap();
    backup
}

#[sqlx::test(migrations = "../../migrations")]
async fn backup_retention_preserves_successful_generations_and_recovers_fenced_deletion(
    pool: PgPool,
) {
    use chrono::{Duration, TimeZone, Utc};
    let mut app = app(pool);
    Arc::make_mut(&mut app.config).development = false;
    let (server, _) = official(&app).await;
    let admin = account(&app, "保存管理者", true).await;
    let player = account(&app, "一般プレイヤー", false).await;
    let first = Utc.with_ymd_and_hms(2026, 8, 21, 20, 0, 0).unwrap();
    let mut pinned = Uuid::nil();
    for day in 0..42 {
        let when = first + Duration::days(day);
        let state = if day >= 40 { "failed" } else { "ready" }; // September 30 and October 1 failed.
        let id = backup_generation(&app, admin.id, server, when, state, true).await;
        if day == 1 {
            pinned = id;
        }
    }
    let manual = backup_generation(
        &app,
        admin.id,
        server,
        first - Duration::days(10),
        "ready",
        false,
    )
    .await;
    assert!(
        commands::execute(
            &app,
            &player,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::BackupPin {
                    id: pinned,
                    pinned: true
                }
            }
        )
        .await
        .is_err()
    );
    run(
        &app,
        &admin,
        Command::BackupPin {
            id: pinned,
            pinned: true,
        },
    )
    .await;
    let now = Utc.with_ymd_and_hms(2026, 10, 1, 20, 0, 0).unwrap();
    for _ in 0..3 {
        lkjmc_core::services::backup_maintenance(&app, now)
            .await
            .unwrap();
    }
    let dates:Vec<String>=sqlx::query_scalar("SELECT scheduled_for::text FROM backups WHERE state='ready' AND scheduled_for IS NOT NULL ORDER BY scheduled_for").fetch_all(&app.db).await.unwrap();
    assert_eq!(
        dates,
        vec![
            "2026-08-22",
            "2026-09-13",
            "2026-09-20",
            "2026-09-23",
            "2026-09-24",
            "2026-09-25",
            "2026-09-26",
            "2026-09-27",
            "2026-09-28",
            "2026-09-29"
        ]
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT state FROM backups WHERE id=$1")
            .bind(manual)
            .fetch_one(&app.db)
            .await
            .unwrap(),
        "ready"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM backups WHERE state='failed'")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        2
    );
    let token = host_token(&app).await;
    let (_, response) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    let job = response["job"].clone();
    assert_eq!(job["kind"], "official.backup.prune");
    let id = id(&job["payload"], "backup_id");
    assert!(
        commands::execute(
            &app,
            &admin,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::BackupPin { id, pinned: true }
            }
        )
        .await
        .is_err(),
        "A generation already being deleted cannot be pinned late"
    );
    let job_id = job["id"].as_str().unwrap();
    let (_, context) = host_http(
        &app,
        &token,
        &format!("/internal/v1/jobs/{job_id}/context"),
        json!({"lease_token":job["lease_token"]}),
    )
    .await;
    assert!(context["rejected"].is_null(), "{context}");
    assert!(
        !sqlx::query_scalar::<_, bool>("SELECT maintenance FROM servers WHERE id=$1")
            .bind(server)
            .fetch_one(&app.db)
            .await
            .unwrap(),
        "Retention must not close the running SMP"
    );
    let ack = format!("/internal/v1/jobs/{job_id}/ack");
    let receipt = json!({"effect":"committed","backup_id":id,"server_id":server,"host_deleted":true,"database_deleted":true});
    assert_eq!(
        host_http(
            &app,
            &token,
            &ack,
            json!({"lease_token":job["lease_token"],"state":"succeeded","result":receipt})
        )
        .await
        .0,
        StatusCode::CONFLICT,
        "Cannot claim completion before Core copy is removed"
    );
    assert_eq!(
        host_http(
            &app,
            &token,
            &ack,
            json!({"lease_token":job["lease_token"],"state":"failed","result":{"effect":"none"}})
        )
        .await
        .0,
        StatusCode::CONFLICT,
        "Uncertain deletion must reconcile, not disappear into failed state"
    );
    let root = app.config.storage.join("official-backups");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(format!("{id}.dump"));
    std::fs::write(&path, b"wrong bytes").unwrap();
    let prune = format!("/internal/v1/jobs/{job_id}/backup-prune");
    assert_eq!(
        host_http(&app, &token, &prune, json!({"lease_token":Uuid::new_v4()}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        host_http(
            &app,
            &token,
            &prune,
            json!({"lease_token":job["lease_token"]})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert!(path.exists());
    std::fs::write(&path, b"retention database fixture").unwrap();
    let (status, result) = host_http(
        &app,
        &token,
        &prune,
        json!({"lease_token":job["lease_token"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert!(!path.exists());
    // Recreate the DB state after durable unlink but before its final transaction.
    sqlx::query("UPDATE backups SET database_pruned_at=NULL WHERE id=$1")
        .bind(id)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(
            &app,
            &token,
            &prune,
            json!({"lease_token":job["lease_token"]})
        )
        .await
        .0,
        StatusCode::OK
    );
    for _ in 0..2 {
        let (status, result) = host_http(
            &app,
            &token,
            &ack,
            json!({"lease_token":job["lease_token"],"state":"succeeded","result":receipt}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{result}");
    }
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT state FROM backups WHERE id=$1")
            .bind(id)
            .fetch_one(&app.db)
            .await
            .unwrap(),
        "pruned"
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT state FROM backups WHERE id=$1")
            .bind(pinned)
            .fetch_one(&app.db)
            .await
            .unwrap(),
        "ready"
    );
    std::fs::remove_dir_all(&app.config.storage).unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn concurrent_manual_official_backups_queue_only_one_save(pool: PgPool) {
    let app = app(pool);
    official(&app).await;
    let first = account(&app, "保存担当一", true).await;
    let second = account(&app, "保存担当二", true).await;
    let (a, b) = tokio::join!(
        commands::execute(
            &app,
            &first,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::OfficialBackup
            }
        ),
        commands::execute(
            &app,
            &second,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::OfficialBackup
            }
        )
    );
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM backups WHERE state='queued'")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        1
    );
}
