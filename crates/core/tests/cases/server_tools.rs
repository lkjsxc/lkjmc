#[sqlx::test(migrations = "../../migrations")]
async fn explicit_file_close_belongs_to_requester_and_is_visible_in_timeline(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "Owner", false).await;
    let editor = account(&app, "Editor", false).await;
    sqlx::query("INSERT INTO trust_ranks VALUES(1,'Fixture host',4,2,8192,4000,102400)")
        .execute(&app.db).await.unwrap();
    sqlx::query("UPDATE accounts SET trust_rank=1 WHERE id=$1")
        .bind(owner.id).execute(&app.db).await.unwrap();
    let server = custom_server(&app, owner.id).await;
    sqlx::query("UPDATE servers SET desired='stopped',observed='stopped' WHERE id=$1")
        .bind(server).execute(&app.db).await.unwrap();
    run(&app, &owner, Command::ServerMember { id: server, member: editor.id, role: Some("administrator".into()) }).await;
    let opened = run(&app, &owner, Command::ServerInspection { id: server, open: true }).await;
    let closed = run(&app, &editor, Command::ServerInspection { id: server, open: false }).await;
    let closing = id(&closed, "job_id");
    let job: Value = sqlx::query_scalar("SELECT to_jsonb(j) FROM jobs j WHERE id=$1")
        .bind(closing).fetch_one(&app.db).await.unwrap();
    assert_eq!(job["actor"], json!(editor.id));
    assert_eq!(job["payload"]["automatic"], false);
    assert_eq!(job["payload"]["inspection"]["id"], opened["job_id"]);
    let (_, timeline) = http(&app, &editor, "GET", "/api/v1/timeline", json!({}), false).await;
    assert!(timeline["items"].as_array().unwrap().iter().any(|item| item["job_id"] == closed["job_id"] && item["open"] == false));
    let (status, details) = http(&app, &editor, "GET", &format!("/api/v1/jobs/{closing}"), json!({}), false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(details["open"], false);
    let (_, owners_timeline) = http(&app, &owner, "GET", "/api/v1/timeline", json!({}), false).await;
    assert!(!owners_timeline["items"].as_array().unwrap().iter().any(|item| item["job_id"] == closed["job_id"]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn server_tools_inspection_lifecycle_reserves_freezes_and_hands_over(pool: PgPool) {
    let app=app(pool);
    let owner=account(&app,"Owner",false).await;
    sqlx::query("INSERT INTO trust_ranks VALUES(1,'Fixture host',4,2,8192,4000,102400)").execute(&app.db).await.unwrap();
    sqlx::query("UPDATE accounts SET trust_rank=1 WHERE id=$1").bind(owner.id).execute(&app.db).await.unwrap();
    let server=custom_server(&app,owner.id).await;
    sqlx::query("UPDATE servers SET desired='stopped',observed='stopped' WHERE id=$1").bind(server).execute(&app.db).await.unwrap();
    let opening=run(&app,&owner,Command::ServerInspection{id:server,open:true}).await;
    let repeat=run(&app,&owner,Command::ServerInspection{id:server,open:true}).await;
    assert_eq!(opening["job_id"],repeat["job_id"]);
    assert!(sqlx::query("UPDATE servers SET memory_mib=memory_mib+512 WHERE id=$1").bind(server).execute(&app.db).await.is_err());
    let token=host_token(&app).await;
    let (_,leased)=host_http(&app,&token,"/internal/v1/poll",json!({})).await;
    let job=&leased["job"];
    assert_eq!(job["id"],opening["job_id"]);
    let context=format!("/internal/v1/jobs/{}/context",id(job,"id"));
    let (_,data)=host_http(&app,&token,&context,json!({"lease_token":job["lease_token"]})).await;
    assert_eq!(data["server"]["inspection_valid"],true);
    let ack=format!("/internal/v1/jobs/{}/ack",id(job,"id"));
    let expiry=chrono::Utc::now().timestamp()+600;
    let result=json!({"effect":"committed","inspection_id":opening["job_id"],"open":true,"guest_ready":true,"game_stopped":true,"expires_unix":expiry});
    assert_eq!(host_http(&app,&token,&ack,json!({"lease_token":job["lease_token"],"state":"succeeded","result":result})).await.0,StatusCode::OK);
    let (_,page)=http(&app,&owner,"GET",&format!("/api/v1/servers/{server}?section=manage-files"),json!({}),false).await;
    assert_eq!(page["server"]["inspection"]["state"],"ready");
    assert_eq!(page["server"]["desired"],"stopped");
    // Starting is one operation and transfers ownership before a stale expiry can stop it.
    let start=run(&app,&owner,Command::ServerStart{id:server}).await;
    assert!(start["job_id"].is_string());
    let stored:Value=sqlx::query_scalar("SELECT to_jsonb(s) FROM servers s WHERE id=$1").bind(server).fetch_one(&app.db).await.unwrap();
    assert_eq!(stored["desired"],"running");
    assert!(stored["inspection"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn server_tools_inspection_revocation_cleans_up_and_owner_is_immutable(pool: PgPool) {
    let app=app(pool);
    let owner=account(&app,"Owner",false).await;
    sqlx::query("INSERT INTO trust_ranks VALUES(1,'Fixture host',4,2,8192,4000,102400)").execute(&app.db).await.unwrap();
    sqlx::query("UPDATE accounts SET trust_rank=1 WHERE id=$1").bind(owner.id).execute(&app.db).await.unwrap();
    let admin=account(&app,"Editor",false).await;
    let server=custom_server(&app,owner.id).await;
    sqlx::query("UPDATE servers SET desired='stopped',observed='stopped' WHERE id=$1").bind(server).execute(&app.db).await.unwrap();
    run(&app,&owner,Command::ServerMember{id:server,member:admin.id,role:Some("administrator".into())}).await;
    let opening=run(&app,&admin,Command::ServerInspection{id:server,open:true}).await;
    run(&app,&owner,Command::ServerMember{id:server,member:admin.id,role:None}).await;
    let token=host_token(&app).await;
    let (_,leased)=host_http(&app,&token,"/internal/v1/poll",json!({})).await;
    assert_eq!(leased["job"]["payload"]["open"],false);
    assert_eq!(leased["job"]["payload"]["inspection"]["id"],opening["job_id"]);
    assert_eq!(leased["job"]["payload"]["automatic"],true);
    let (_,timeline)=http(&app,&admin,"GET","/api/v1/timeline",json!({}),false).await;
    assert!(!timeline["items"].as_array().unwrap().iter().any(|item|item["job_id"]==leased["job"]["id"]));
    let job=&leased["job"];
    let context=format!("/internal/v1/jobs/{}/context",id(job,"id"));
    let (_,data)=host_http(&app,&token,&context,json!({"lease_token":job["lease_token"]})).await;
    assert_eq!(data["server"]["inspection_valid"],false);
    assert!(data["rejected"].is_null(),"cleanup retains original window authority after revocation");
    let ack=format!("/internal/v1/jobs/{}/ack",id(job,"id"));
    assert_eq!(host_http(&app,&token,&ack,json!({"lease_token":job["lease_token"],"state":"succeeded","result":{"effect":"committed","inspection_id":opening["job_id"],"open":false,"game_stopped":true,"guest_ready":false}})).await.0,StatusCode::OK);
    let i:Option<Value>=sqlx::query_scalar("SELECT inspection FROM servers WHERE id=$1").bind(server).fetch_one(&app.db).await.unwrap();
    assert!(i.is_none());
    for role in [None,Some("guest".into())] {
        assert!(commands::execute(&app,&owner,Request{request_id:Uuid::new_v4(),command:Command::ServerMember{id:server,member:owner.id,role}}).await.is_err());
    }
    let (_,page)=http(&app,&owner,"GET",&format!("/api/v1/servers/{server}?section=manage-members"),json!({}),false).await;
    let members=page["servers"][0]["members"].as_array().unwrap();
    assert_eq!(members.len(),1);
    assert_eq!(members[0]["is_owner"],true);
}

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
            path: "config/paper-global.yml".into(),
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
    // The owner is now a separate immutable row; locate the requested member by identity.
    let members=page["servers"][0]["members"].as_array().unwrap();
    let displayed=members.iter().find(|m| m["account_id"]==json!(member.id)).unwrap();
    assert_eq!(displayed["role"], "guest");
    assert_eq!(displayed["minecraft_operator_job"]["operator"], true);
    assert_eq!(displayed["minecraft_identity"]["ready"], false);
    assert_eq!(members.iter().filter(|m|m["account_id"]==json!(owner.id)).count(),1);
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
