#[sqlx::test(migrations = "../../migrations")]
async fn console_unknown_delivery_is_terminal_verified_and_releases_server(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Console owner", false).await;
    let server = custom_server(&app, owner.id).await;
    let token = host_token(&app).await;
    let submitted = run(
        &app,
        &owner,
        Command::ServerConsole {
            id: server,
            line: "say once".into(),
        },
    )
    .await;
    let (_, response) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    let job = &response["job"];
    assert_eq!(job["id"], submitted["job_id"]);
    let job_id = id(job, "id");
    let (_, context) = host_http(
        &app,
        &token,
        &format!("/internal/v1/jobs/{job_id}/context"),
        json!({"lease_token":job["lease_token"]}),
    )
    .await;
    assert!(context["job"]["host_authorized_at"].is_string());
    let route = format!("/internal/v1/jobs/{job_id}/ack");
    let receipt = json!({"effect":"uncertain","prepared_receipt":true,"job_id":job_id,
        "command_sha256":auth::hash("say once\n"),"authorized_at":context["job"]["host_authorized_at"],
        "prepared_lease_token":job["lease_token"]});
    let mut wrong = receipt.clone();
    wrong["command_sha256"] = json!(auth::hash("say twice\n"));
    assert_eq!(
        host_http(
            &app,
            &token,
            &route,
            json!({"lease_token":job["lease_token"],"state":"delivery_unknown","result":wrong})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    sqlx::query("UPDATE jobs SET lease_until=now()-interval '1 second' WHERE id=$1")
        .bind(job_id)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(
            &app,
            &token,
            &route,
            json!({"lease_token":job["lease_token"],"state":"leased"})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    // Expiry ends renewal authority; the same-generation durable outcome can settle.
    assert_eq!(
        host_http(
            &app,
            &token,
            &route,
            json!({"lease_token":job["lease_token"],"state":"delivery_unknown","result":receipt})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        host_http(
            &app,
            &token,
            &route,
            json!({"lease_token":job["lease_token"],"state":"delivery_unknown","result":receipt})
        )
        .await
        .0,
        StatusCode::OK
    );
    let row: (bool, Option<Uuid>) =
        sqlx::query_as("SELECT maintenance,maintenance_job_id FROM servers WHERE id=$1")
            .bind(server)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(row, (false, None));
    let read = run(
        &app,
        &owner,
        Command::ServerLogs {
            id: server,
            date: None,
        },
    )
    .await;
    let (_, polled) = host_http(&app, &token, "/internal/v1/poll", json!({"lane":"read"})).await;
    assert_eq!(polled["job"]["id"], read["job_id"]);
    let newer = run(
        &app,
        &owner,
        Command::ServerConsole {
            id: server,
            line: "say deliberate new action".into(),
        },
    )
    .await;
    assert_ne!(newer["job_id"], submitted["job_id"]);
    let (_, polled) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    assert_eq!(polled["job"]["id"], newer["job_id"]);
    let (_, details) = http(
        &app,
        &owner,
        "GET",
        &format!("/api/v1/jobs/{job_id}"),
        json!({}),
        false,
    )
    .await;
    assert_eq!(details["operation_status"]["terminal"], true);
    assert_eq!(details["operation_status"]["automatic_retry"], false);
    let (_, timeline) = http(&app, &owner, "GET", "/api/v1/timeline", json!({}), false).await;
    let item = timeline["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["job_id"] == json!(job_id))
        .unwrap();
    assert_eq!(item["operation_status"]["outcome"], "delivery_unknown");
}

#[sqlx::test(migrations = "../../migrations")]
async fn read_lane_is_bounded_independent_and_excludes_restore(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Read owner", false).await;
    let servers = [
        custom_server(&app, owner.id).await,
        custom_server(&app, owner.id).await,
        custom_server(&app, owner.id).await,
    ];
    let token = host_token(&app).await;
    let backup = run(&app, &owner, Command::ServerBackup { id: servers[0] }).await;
    let (_, response) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    let backing = &response["job"];
    assert_eq!(backing["id"], backup["job_id"]);
    host_http(
        &app,
        &token,
        &format!(
            "/internal/v1/jobs/{}/context",
            backing["id"].as_str().unwrap()
        ),
        json!({"lease_token":backing["lease_token"]}),
    )
    .await;
    for server in servers {
        run(
            &app,
            &owner,
            Command::ServerLogs {
                id: server,
                date: None,
            },
        )
        .await;
    }
    let (_, one) = host_http(&app, &token, "/internal/v1/poll", json!({"lane":"read"})).await;
    assert_eq!(one["job"]["server_id"], json!(servers[0]));
    let (_, two) = host_http(&app, &token, "/internal/v1/poll", json!({"lane":"read"})).await;
    let (_, full) = host_http(&app, &token, "/internal/v1/poll", json!({"lane":"read"})).await;
    assert!(full["job"].is_null());
    let restore = Uuid::new_v4();
    sqlx::query("INSERT INTO jobs(id,actor,server_id,worker,kind,payload) VALUES($1,$2,$3,'host','server.restore','{}')")
        .bind(restore).bind(owner.id).bind(id(&two["job"], "server_id")).execute(&app.db).await.unwrap();
    let (_, blocked) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    assert!(blocked["job"].is_null());
    let read = &two["job"];
    assert_eq!(host_http(&app, &token, &format!("/internal/v1/jobs/{}/ack", read["id"].as_str().unwrap()),
        json!({"lease_token":read["lease_token"],"state":"succeeded","result":{"lines":[],"date":null}})).await.0, StatusCode::OK);
    let (_, restoring) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    assert_eq!(restoring["job"]["id"], json!(restore));
    let next = run(
        &app,
        &owner,
        Command::ServerLogs {
            id: id(read, "server_id"),
            date: Some("2026-10-04".into()),
        },
    )
    .await;
    let (_, next_read) = host_http(&app, &token, "/internal/v1/poll", json!({"lane":"read"})).await;
    assert_ne!(next_read["job"]["id"], next["job_id"]);
    assert_eq!(next_read["job"]["server_id"], json!(servers[2]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn machine_observations_do_not_manufacture_game_readiness(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "State owner", false).await;
    let server = custom_server(&app, owner.id).await;
    sqlx::query("UPDATE servers SET observed='stopped',players=3 WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    let token = host_token(&app).await;
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/observations",
            json!({"server_id":server,"machine_observed":"running"})
        )
        .await
        .0,
        StatusCode::OK
    );
    let data: Value = sqlx::query_scalar("SELECT to_jsonb(s) FROM servers s WHERE id=$1")
        .bind(server)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(data["observed"], "stopped");
    assert_eq!(data["players"], 3);
    let status = serde_json::to_value(lkjmc_core::server_tools::server_status(&data)).unwrap();
    assert_eq!(status["machine_state"], "running");
    assert_eq!(status["game_state"], "stopped");
    assert_eq!(status["joinable"], false);
    assert_eq!(status["actions"]["join"]["allowed"], true);
    assert_eq!(
        internal(
            &app,
            "official",
            Some(server),
            "/internal/v1/observations",
            json!({"server_id":server,"machine_observed":"running"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}
