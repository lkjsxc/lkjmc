#[sqlx::test(migrations = "../../migrations")]
async fn admin_operations_filter_counts_and_page_equal_timestamps_without_read_noise(pool: PgPool) {
    let app = app(pool);
    let admin = account(&app, "Operations admin", true).await;
    let other_admin = account(&app, "Other admin", true).await;
    let player = account(&app, "Player", false).await;
    let (server, _) = official(&app).await;
    for state in [
        "queued",
        "leased",
        "waiting",
        "failed",
        "delivery_unknown",
        "succeeded",
        "cancelled",
    ] {
        let repetitions = if state == "queued" { 52 } else { 1 };
        for _ in 0..repetitions {
            sqlx::query("INSERT INTO jobs(id,actor,server_id,worker,kind,state,payload,error,progress,created_at) VALUES($1,$2,$3,'host',$4,$5,'{}',$6,$7,'2026-10-04T00:00:00Z')")
                .bind(Uuid::new_v4()).bind(player.id).bind(server)
                .bind(if state=="delivery_unknown" {"server.console"} else {"fixture.operation"}).bind(state)
                .bind((state == "failed").then(||json!({"id":"error.internal","params":{"reference":"operations-test"}}).to_string()))
                .bind(json!({"message":{"id":"text.waiting_to_resume","params":{}}}))
                .execute(&app.db).await.unwrap();
        }
        // Only console sends can have an uncertain delivery outcome.
        if state == "delivery_unknown" {
            continue;
        }
        for kind in ["server.files", "server.file.read", "server.logs"] {
            sqlx::query("INSERT INTO jobs(id,actor,server_id,worker,kind,state,payload) VALUES($1,$2,$3,'host',$4,$5,'{}')")
                .bind(Uuid::new_v4()).bind(player.id).bind(server).bind(kind).bind(state)
                .execute(&app.db).await.unwrap();
        }
    }
    assert_eq!(
        http(
            &app,
            &player,
            "GET",
            "/api/v1/admin/operations",
            json!({}),
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut path = "/api/v1/admin/operations".to_string();
    let mut seen = std::collections::BTreeSet::new();
    let mut first_cursor = None;
    let mut previous = None;
    loop {
        let (status, page) = http(&app, &admin, "GET", &path, json!({}), false).await;
        assert_eq!(status, StatusCode::OK, "{page}");
        assert_eq!(page["filter"], "active");
        assert_eq!(page["counts"], json!({"active":54,"failed":2,"history":4}));
        let rows = page["operations"].as_array().unwrap();
        assert!(rows.len() <= 25);
        for row in rows {
            assert_eq!(row["server_id"], server.to_string());
            assert!(row["server_name"].is_string());
            assert_eq!(row["progress"]["message"]["id"], "text.waiting_to_resume");
            assert!(row.get("payload").is_none());
            assert!(row.get("result").is_none());
            assert!(matches!(
                row["state"].as_str(),
                Some("queued" | "leased" | "waiting")
            ));
            let id = Uuid::parse_str(row["id"].as_str().unwrap()).unwrap();
            assert!(previous.is_none_or(|last| id < last));
            previous = Some(id);
            assert!(seen.insert(id));
        }
        let Some(cursor) = page["next_cursor"].as_str() else {
            break;
        };
        first_cursor.get_or_insert_with(|| cursor.to_owned());
        path = format!("/api/v1/admin/operations?cursor={cursor}");
    }
    assert_eq!(seen.len(), 54);
    let first_cursor = first_cursor.unwrap();
    for (viewer, path) in [
        (
            &other_admin,
            format!("/api/v1/admin/operations?cursor={first_cursor}"),
        ),
        (
            &admin,
            format!("/api/v1/admin/operations?filter=failed&cursor={first_cursor}"),
        ),
        (&admin, "/api/v1/admin/operations?cursor=invalid".into()),
        (
            &admin,
            format!("/api/v1/admin/operations?cursor={}", "x".repeat(513)),
        ),
        (&admin, "/api/v1/admin/operations?filter=unknown".into()),
    ] {
        assert_eq!(
            http(&app, viewer, "GET", &path, json!({}), false).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    for (filter, length) in [("failed", 2), ("history", 4)] {
        let (status, page) = http(
            &app,
            &admin,
            "GET",
            &format!("/api/v1/admin/operations?filter={filter}"),
            json!({}),
            false,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{page}");
        assert_eq!(page["operations"].as_array().unwrap().len(), length);
        let failed = page["operations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["state"] == "failed")
            .unwrap();
        assert_eq!(failed["error"]["id"], "error.internal");
        assert_eq!(failed["error"]["params"]["reference"], "operations-test");
    }
    sqlx::query("UPDATE jobs SET error='private historical diagnostic' WHERE kind='fixture.operation' AND state='failed'")
        .execute(&app.db).await.unwrap();
    let (_, failed_page) = http(
        &app,
        &admin,
        "GET",
        "/api/v1/admin/operations?filter=failed",
        json!({}),
        false,
    )
    .await;
    let failed = failed_page["operations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["state"] == "failed")
        .unwrap();
    assert_eq!(failed["error"]["id"], "system.unknown");
    assert_eq!(failed["error"]["params"]["reference"], failed["id"]);
    assert!(
        !failed_page
            .to_string()
            .contains("private historical diagnostic")
    );
    let (status, overview) = http(
        &app,
        &admin,
        "GET",
        "/api/v1/view/admin?section=overview",
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{overview}");
    assert_eq!(overview["counts"]["operations"], 54);
    assert!(overview["counts"].get("jobs").is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn hosting_allowance_uses_ownership_and_reserves_running_games_and_file_guests(pool: PgPool) {
    let mut app = app(pool);
    Arc::make_mut(&mut app.config).development = false;
    let owner = account(&app, "Server owner", false).await;
    let shared_owner = account(&app, "Shared server owner", false).await;
    sqlx::query("INSERT INTO trust_ranks(id,name,server_count,concurrent_servers,memory_mib,cpu_millis,storage_mib) VALUES(90,'Quota fixture',4,2,8192,4000,65536)").execute(&app.db).await.unwrap();
    sqlx::query("UPDATE accounts SET trust_rank=90 WHERE id=$1")
        .bind(owner.id)
        .execute(&app.db)
        .await
        .unwrap();
    let mut owned = vec![];
    for n in 0..3 {
        let server = Uuid::new_v4();
        sqlx::query("INSERT INTO servers(id,owner,name,kind,visibility,version,software,memory_mib,cpu_millis,storage_mib,desired,inspection) VALUES($1,$2,$3,'custom','private','test','custom',2048,1000,16384,$4,$5)")
            .bind(server).bind(owner.id).bind(format!("Owned {n}"))
            .bind(if n==0 {"running"}else{"stopped"})
            .bind((n==1).then(||json!({"fixture":true}))).execute(&app.db).await.unwrap();
        owned.push(server);
    }
    let shared = Uuid::new_v4();
    sqlx::query("INSERT INTO servers(id,owner,name,kind,visibility,version,software,memory_mib,cpu_millis,storage_mib,desired) VALUES($1,$2,'Shared','custom','private','test','custom',4096,2000,32768,'running')")
        .bind(shared).bind(shared_owner.id).execute(&app.db).await.unwrap();
    sqlx::query(
        "INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,'administrator')",
    )
    .bind(shared)
    .bind(owner.id)
    .execute(&app.db)
    .await
    .unwrap();
    let (status, presets) = http(
        &app,
        &owner,
        "GET",
        "/api/v1/server-presets",
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{presets}");
    let quota = &presets["hosting"];
    assert_eq!(
        quota["owned"],
        json!({"server_count":3,"storage_mib":49152})
    );
    assert_eq!(
        quota["reserved"],
        json!({"server_count":2,"memory_mib":4096,"cpu_millis":2000})
    );
    assert_eq!(
        quota["remaining"],
        json!({"server_count":1,"storage_mib":16384,"concurrent_servers":0,"memory_mib":4096,"cpu_millis":2000})
    );
    assert_eq!(quota["minimum_server_storage_mib"], 16384);
    assert_eq!(quota["minimum_server_cpu_millis"], 1000);
    // Creation is allowed even while all running slots are reserved.
    assert_eq!(quota["can_create"], true);
    let (status, servers) = http(
        &app,
        &owner,
        "GET",
        "/api/v1/view/servers",
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{servers}");
    assert_eq!(&servers["hosting"], quota);
    assert_eq!(servers["servers"].as_array().unwrap().len(), 4);
    sqlx::query("UPDATE servers SET storage_mib=32768 WHERE id=$1")
        .bind(owned[2])
        .execute(&app.db)
        .await
        .unwrap();
    let (_, page) = http(
        &app,
        &owner,
        "GET",
        "/api/v1/server-presets",
        json!({}),
        false,
    )
    .await;
    assert_eq!(page["hosting"]["can_create"], false);
    assert_eq!(
        page["hosting"]["creation_blocked_reason"]["id"],
        "text.hosting_storage_allowance_insufficient"
    );
    let (_, page) = http(
        &app,
        &shared_owner,
        "GET",
        "/api/v1/server-presets",
        json!({}),
        false,
    )
    .await;
    assert_eq!(page["hosting"]["can_create"], false);
    assert_eq!(
        page["hosting"]["creation_blocked_reason"]["id"],
        "text.hosting_server_limit_reached"
    );
}
