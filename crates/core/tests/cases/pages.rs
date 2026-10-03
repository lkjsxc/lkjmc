#[sqlx::test(migrations = "../../migrations")]
async fn histories_are_bounded_private_and_cursor_ordered(pool: PgPool) {
    let app = app(pool);
    let a = account(&app, "Reader", false).await;
    let b = account(&app, "Other", false).await;
    for owner in [a.id, b.id] {
        for n in 0..61 {
            sqlx::query("INSERT INTO notifications(account_id,kind,body) VALUES($1,'message',$2)")
                .bind(owner)
                .bind(json!({"n":n}))
                .execute(&app.db)
                .await
                .unwrap();
        }
    }
    let (status, home) = http(&app, &a, "GET", "/api/v1/home", json!({}), false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(home["notifications"].as_array().unwrap().len(), 3);
    assert_eq!(home["counts"]["notifications"], 61);
    let mut path = "/api/v1/history/notifications?unread=true".to_string();
    let mut seen = std::collections::BTreeSet::new();
    let mut previous = i64::MAX;
    loop {
        let (status, page) = http(&app, &a, "GET", &path, json!({}), false).await;
        assert_eq!(status, StatusCode::OK);
        let rows = page["notifications"].as_array().unwrap();
        assert!(rows.len() <= 25);
        for row in rows {
            assert_eq!(row["account_id"], a.id.to_string());
            let id = row["id"].as_i64().unwrap();
            assert!(id < previous);
            previous = id;
            assert!(seen.insert(id));
        }
        let Some(cursor) = page["next_cursor"].as_str() else {
            break;
        };
        path = format!("/api/v1/history/notifications?unread=true&cursor={cursor}");
    }
    assert_eq!(seen.len(), 61);
    let (status, _) = http(
        &app,
        &a,
        "GET",
        "/api/v1/history/activity?cursor=bad",
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    for _ in 0..51 {
        sqlx::query("INSERT INTO jobs(id,actor,worker,kind,payload,state,created_at) VALUES($1,$2,'host','server.start','{}','succeeded','2026-10-03T00:00:00Z')").bind(Uuid::new_v4()).bind(a.id).execute(&app.db).await.unwrap();
        sqlx::query("INSERT INTO invitations(id,sender,recipient,kind,resource_id,created_at) VALUES($1,$2,$3,'room',$4,'2026-10-03T00:00:00Z')").bind(Uuid::new_v4()).bind(b.id).bind(a.id).bind(Uuid::new_v4()).execute(&app.db).await.unwrap();
    }
    for kind in ["activity", "invitations"] {
        let mut path = format!("/api/v1/history/{kind}");
        let mut seen = std::collections::BTreeSet::new();
        loop {
            let (status, page) = http(&app, &a, "GET", &path, json!({}), false).await;
            assert_eq!(status, StatusCode::OK, "{page}");
            let key = if kind == "activity" { "jobs" } else { kind };
            let rows = page[key].as_array().unwrap();
            assert!(rows.len() <= 25);
            for row in rows {
                assert!(seen.insert(row["id"].as_str().unwrap().to_string()));
            }
            let Some(cursor) = page["next_cursor"].as_str() else {
                break;
            };
            path = format!("/api/v1/history/{kind}?cursor={cursor}");
        }
        assert_eq!(seen.len(), 51);
    }
    // A selected social page contains no unrelated private data.
    let (_, social) = http(
        &app,
        &a,
        "GET",
        "/api/v1/view/social?section=friends",
        json!({}),
        false,
    )
    .await;
    assert_eq!(
        social.as_object().unwrap().keys().collect::<Vec<_>>(),
        vec!["friends"]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn server_pages_enforce_each_permission_and_hide_unrelated_payloads(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Owner", false).await;
    let operator = account(&app, "Operator", false).await;
    let stranger = account(&app, "Stranger", false).await;
    let (server, _) = official(&app).await;
    sqlx::query("UPDATE servers SET owner=$2,visibility='private',last_observed_at=now()-interval '2 minutes' WHERE id=$1").bind(server).bind(owner.id).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,'operator')")
        .bind(server)
        .bind(operator.id)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        http(
            &app,
            &stranger,
            "GET",
            &format!("/api/v1/servers/{server}"),
            json!({}),
            false
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (_, page) = http(
        &app,
        &operator,
        "GET",
        &format!("/api/v1/servers/{server}?section=manage-overview"),
        json!({}),
        false,
    )
    .await;
    assert_eq!(page["server"]["observed"], "unknown");
    assert_eq!(page["server"]["can_administer"], false);
    assert!(page["servers"][0].get("artifacts").is_none());
    for section in ["files", "backups", "members", "settings"] {
        assert_eq!(
            http(
                &app,
                &operator,
                "GET",
                &format!("/api/v1/servers/{server}?section=manage-{section}"),
                json!({}),
                false
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
    }
    let (_, page) = http(
        &app,
        &owner,
        "GET",
        &format!("/api/v1/servers/{server}?section=manage-files"),
        json!({}),
        false,
    )
    .await;
    assert!(page["servers"][0]["artifacts"].is_array());
    assert!(page["servers"][0].get("backups").is_none());
    assert_eq!(
        http(
            &app,
            &owner,
            "GET",
            &format!("/api/v1/servers/{server}?section=manage-missing"),
            json!({}),
            false
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn unavailable_presets_leave_no_server_or_job_and_pinned_presets_create(pool: PgPool) {
    let mut app = app(pool);
    Arc::make_mut(&mut app.config).development = false;
    let a = account(&app, "Builder", true).await;
    sqlx::query("INSERT INTO trust_ranks(id,name,server_count,concurrent_servers,memory_mib,cpu_millis,storage_mib) VALUES(99,'Test',3,2,8192,4000,40960)").execute(&app.db).await.unwrap();
    sqlx::query("UPDATE accounts SET trust_rank=99 WHERE id=$1")
        .bind(a.id)
        .execute(&app.db)
        .await
        .unwrap();
    let create = |software: &str, version: &str| {
        serde_json::from_value::<Command>(json!({"type":"server_create","name":"Test","software":software,"version":version,"memory_mib":2048,"cpu_millis":1000,"storage_mib":16384,"visibility":"private","community":null})).unwrap()
    };
    let unavailable = commands::execute(
        &app,
        &a,
        Request {
            request_id: Uuid::new_v4(),
            command: create("paper", "1.21.11"),
        },
    )
    .await;
    assert!(unavailable.is_err());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM servers")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM jobs")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        0
    );
    app.presets=Arc::new(vec![serde_json::from_value(json!({"software":"paper","version":"1.21.11","java":21,"url":"https://example.com/paper.jar","sha256":"a".repeat(64)})).unwrap()]);
    let (_, presets) = http(&app, &a, "GET", "/api/v1/server-presets", json!({}), false).await;
    assert_eq!(presets["presets"][0]["java"], 21);
    assert_eq!(presets["minimum_storage_mib"], 16384);
    assert!(presets["presets"][0].get("url").is_none());
    let mut too_small = serde_json::to_value(create("paper", "1.21.11")).unwrap();
    too_small["storage_mib"] = json!(10240);
    assert!(
        commands::execute(
            &app,
            &a,
            Request {
                request_id: Uuid::new_v4(),
                command: serde_json::from_value(too_small).unwrap()
            }
        )
        .await
        .is_err()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM servers")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM jobs")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        0
    );
    let accepted = run(&app, &a, create("paper", "1.21.11")).await;
    assert!(accepted["job_id"].is_string());
    let custom = run(&app, &a, create("custom", "1.21.11")).await;
    assert!(custom["job_id"].is_string());
}
