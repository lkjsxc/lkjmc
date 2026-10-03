#[sqlx::test(migrations = "../../migrations")]
async fn server_tools_reads_coalesce_prioritize_and_revoke(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Owner", false).await;
    let member = account(&app, "Admin", false).await;
    let server = custom_server(&app, owner.id).await;
    run(
        &app,
        &owner,
        Command::ServerMember {
            id: server,
            member: member.id,
            role: Some("administrator".into()),
        },
    )
    .await;
    let read = run(
        &app,
        &member,
        Command::ServerFiles {
            id: server,
            path: "".into(),
        },
    )
    .await;
    for _ in 0..130 {
        assert_eq!(
            run(
                &app,
                &member,
                Command::ServerFiles {
                    id: server,
                    path: "".into()
                }
            )
            .await["job_id"],
            read["job_id"]
        );
    }
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM idempotency WHERE actor=$1")
        .bind(member.id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(count, 0);
    let stop = run(&app, &owner, Command::ServerStop { id: server }).await;
    let token = host_token(&app).await;
    let (_, polled) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    assert_eq!(polled["job"]["id"], stop["job_id"]);
    // Save a fixture read result and then revoke permission. Original authors cannot fetch old content.
    let job = id(&read, "job_id");
    sqlx::query("UPDATE jobs SET state='succeeded',result='{\"text\":\"private\"}' WHERE id=$1")
        .bind(job)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        http(
            &app,
            &member,
            "GET",
            &format!("/api/v1/jobs/{job}"),
            json!({}),
            false
        )
        .await
        .0,
        StatusCode::OK
    );
    run(
        &app,
        &owner,
        Command::ServerMember {
            id: server,
            member: member.id,
            role: None,
        },
    )
    .await;
    assert_eq!(
        http(
            &app,
            &member,
            "GET",
            &format!("/api/v1/jobs/{job}"),
            json!({}),
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    sqlx::query("UPDATE jobs SET updated_at=now()-interval '3 minutes' WHERE id=$1")
        .bind(job)
        .execute(&app.db)
        .await
        .unwrap();
    let mut db = app.db.acquire().await.unwrap();
    lkjmc_core::server_tools::prune(&mut db).await.unwrap();
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM jobs WHERE id=$1)")
        .bind(job)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert!(!exists);
}

#[sqlx::test(migrations = "../../migrations")]
async fn server_tools_paths_dates_permissions_and_stopped_mutations(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Owner", false).await;
    let legacy = account(&app, "Legacy", false).await;
    let server = custom_server(&app, owner.id).await;
    run(
        &app,
        &owner,
        Command::ServerMember {
            id: server,
            member: legacy.id,
            role: Some("operator".into()),
        },
    )
    .await;
    for command in [
        Command::ServerFileWrite {
            id: server,
            path: "notes.txt".into(),
            text: "x".into(),
            expected_sha256: None,
        },
        Command::ServerLogs {
            id: server,
            date: Some("2026-02-30".into()),
        },
        Command::ServerFiles {
            id: server,
            path: "config".into(),
        },
        Command::ServerFileRead {
            id: server,
            path: "server.properties".into(),
        },
        Command::ServerFileDelete {
            id: server,
            path: "notes.txt".into(),
            expected_sha256: "bad".into(),
        },
    ] {
        assert!(
            commands::execute(
                &app,
                &owner,
                Request {
                    request_id: Uuid::new_v4(),
                    command
                }
            )
            .await
            .is_err()
        );
    }
    assert!(
        commands::execute(
            &app,
            &legacy,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::ServerFiles {
                    id: server,
                    path: "".into()
                }
            }
        )
        .await
        .is_err()
    );
    run(
        &app,
        &legacy,
        Command::ServerLogs {
            id: server,
            date: None,
        },
    )
    .await; // existing limited host role remains limited
    sqlx::query("UPDATE servers SET desired='stopped',observed='stopped' WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    let write = run(
        &app,
        &owner,
        Command::ServerFileWrite {
            id: server,
            path: "notes.txt".into(),
            text: "hello".into(),
            expected_sha256: None,
        },
    )
    .await;
    let token = host_token(&app).await;
    let (_, polled) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    assert_eq!(polled["job"]["id"], write["job_id"]);
    let route = format!(
        "/internal/v1/jobs/{}/context",
        write["job_id"].as_str().unwrap()
    );
    let (_, context) = host_http(
        &app,
        &token,
        &route,
        json!({"lease_token":polled["job"]["lease_token"]}),
    )
    .await;
    assert_eq!(context["job"]["payload"]["text"], "hello");
    let authorized: bool =
        sqlx::query_scalar("SELECT host_authorized_at IS NOT NULL FROM jobs WHERE id=$1")
            .bind(id(&write, "job_id"))
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert!(authorized);
    let op_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM jobs WHERE kind='server.operator'")
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(op_count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn server_tools_native_op_requires_proven_identity_and_rechecks_at_effect(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Owner", false).await;
    let member = account(&app, "Player", false).await;
    let server = custom_server(&app, owner.id).await;
    run(
        &app,
        &owner,
        Command::ServerMember {
            id: server,
            member: member.id,
            role: Some("guest".into()),
        },
    )
    .await;
    sqlx::query("UPDATE servers SET desired='stopped',observed='stopped' WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    let make = || Command::ServerOperator {
        id: server,
        member: member.id,
        operator: true,
    };
    assert!(
        commands::execute(
            &app,
            &owner,
            Request {
                request_id: Uuid::new_v4(),
                command: make()
            }
        )
        .await
        .is_err()
    );
    let native = Uuid::new_v4();
    sqlx::query("INSERT INTO identities(issuer,subject,account_id,display_name) VALUES('java',$1,$2,'Player')").bind(native.to_string()).bind(member.id).execute(&app.db).await.unwrap();
    sqlx::query("UPDATE profiles SET native_uuid=$2 WHERE account_id=$1 AND status='active'")
        .bind(member.id)
        .bind(native)
        .execute(&app.db)
        .await
        .unwrap();
    let op = run(&app, &owner, make()).await;
    let token = host_token(&app).await;
    let (_, polled) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    assert_eq!(polled["job"]["id"], op["job_id"]);
    let route = format!(
        "/internal/v1/jobs/{}/context",
        op["job_id"].as_str().unwrap()
    );
    let body = json!({"lease_token":polled["job"]["lease_token"]});
    let (_, context) = host_http(&app, &token, &route, body.clone()).await;
    assert_eq!(
        context["job"]["payload"]["identity"]["uuid"],
        native.to_string()
    );
    // Even after initial durable host authorization, an OP retry must recheck identity.
    sqlx::query("DELETE FROM identities WHERE account_id=$1")
        .bind(member.id)
        .execute(&app.db)
        .await
        .unwrap();
    let (_, rejected) = host_http(&app, &token, &route, body).await;
    assert_eq!(rejected["effect"], "uncertain");
    assert!(rejected["rejected"].is_string());
    let (_, page) = http(
        &app,
        &owner,
        "GET",
        &format!("/api/v1/servers/{server}?section=manage-members"),
        json!({}),
        false,
    )
    .await;
    // The native state is an explicit job, never inferred from the guest membership role.
    assert_eq!(page["servers"][0]["members"][0]["role"], "guest");
    assert_eq!(
        page["servers"][0]["members"][0]["minecraft_operator_job"]["operator"],
        true
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn server_tools_settlement_validates_effect_and_bounds_read_retention(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Owner", false).await;
    let server = custom_server(&app, owner.id).await;
    let token = host_token(&app).await;
    let read = run(
        &app,
        &owner,
        Command::ServerFileRead {
            id: server,
            path: "notes.txt".into(),
        },
    )
    .await;
    let (_, polled) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    let route = format!("/internal/v1/jobs/{}/ack", read["job_id"].as_str().unwrap());
    let mut result = json!({"path":"notes.txt","text":"hello","bytes":5,"sha256":"wrong"});
    let bad = host_http(
        &app,
        &token,
        &route,
        json!({"lease_token":polled["job"]["lease_token"],"state":"succeeded","result":result}),
    )
    .await;
    assert_eq!(bad.0, StatusCode::BAD_REQUEST);
    result["sha256"] = json!(auth::hash("hello"));
    let good = host_http(
        &app,
        &token,
        &route,
        json!({"lease_token":polled["job"]["lease_token"],"state":"succeeded","result":result}),
    )
    .await;
    assert_eq!(good.0, StatusCode::OK);
    let notifications: i64 =
        sqlx::query_scalar("SELECT count(*) FROM notifications WHERE account_id=$1")
            .bind(owner.id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(notifications, 0);
    for _ in 0..150 {
        sqlx::query("INSERT INTO jobs(id,actor,server_id,worker,kind,payload,state) VALUES($1,$2,$3,'host','server.files','{}','succeeded')").bind(Uuid::new_v4()).bind(owner.id).bind(server).execute(&app.db).await.unwrap();
    }
    let mut db = app.db.acquire().await.unwrap();
    lkjmc_core::server_tools::prune(&mut db).await.unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM jobs WHERE kind IN ('server.logs','server.files','server.file.read')",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(count, 128);
    for path in ["a.txt", "b.txt", "c.txt"] {
        run(
            &app,
            &owner,
            Command::ServerFileRead {
                id: server,
                path: path.into(),
            },
        )
        .await;
    }
    assert!(
        commands::execute(
            &app,
            &owner,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::ServerFileRead {
                    id: server,
                    path: "d.txt".into()
                }
            }
        )
        .await
        .is_err()
    );
    run(&app, &owner, Command::ServerStop { id: server }).await; // bounded reads never use the action allowance
}
