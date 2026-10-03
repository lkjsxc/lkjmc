// Dedicated real PostgreSQL tests. No host worker or Minecraft process is required.
async fn join_proxy(app: &App) -> String {
    let token = auth::random_token();
    sqlx::query("INSERT INTO service_credentials(id,name,token_hash,role) VALUES($1,'join-fixture',$2,'proxy')")
        .bind(Uuid::new_v4()).bind(auth::hash(&token)).execute(&app.db).await.unwrap();
    token
}
async fn join_server(app: &App, kind: &str, observed: &str) -> Uuid {
    let server = Uuid::new_v4();
    sqlx::query("INSERT INTO servers(id,name,kind,visibility,desired,observed,version,software,memory_mib,cpu_millis,storage_mib,last_observed_at,capabilities) VALUES($1,$2,$2,'public','stopped',$3,'fixture','paper',2048,2000,10240,now(),$4)")
        .bind(server).bind(kind).bind(observed).bind(json!({"proxy_join":true,"bedrock":true})).execute(&app.db).await.unwrap();
    server
}
async fn join_connect(app: &App, token: &str, native: Uuid, client: &str) -> Value {
    let (status, player) = host_http(app, token, "/internal/v1/game/connect", json!({
        "issuer":client,"subject":if client=="java" {native.to_string()} else {native.as_u128().to_string()},
        "native_uuid":native,"session_id":Uuid::new_v4(),"display_name":"Traveller"
    })).await;
    assert_eq!(status, StatusCode::OK, "{player}");
    player
}
async fn join_submit(app: &App, token: &str, player: &Value, server: Uuid, request: Uuid) -> Value {
    let (status, response) = host_http(app, token, "/internal/v1/game/command", json!({
        "account_id":player["account_id"],"session_id":player["session_id"],"request_id":request,
        "command":{"type":"server_join","id":server}
    })).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    response["result"].clone()
}
fn join_request(player: &Value, job: &Value, phase: &str) -> Value {
    json!({"account_id":player["account_id"],"session_id":player["session_id"],
        "server_id":job["server_id"],"join_job_id":job["id"],"lease_token":job["lease_token"],"join_phase":phase})
}
async fn join_lease(app: &App, token: &str) -> Value {
    let (status, response) = host_http(app, token, "/internal/v1/poll", json!({})).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    assert!(!response["job"].is_null(), "expected a join job");
    response["job"].clone()
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_binding_coalescing_supersession_and_observed_success(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let player = join_connect(&app, &token, Uuid::new_v4(), "java").await;
    let sleeping = join_server(&app, "official", "stopped").await;
    let lobby = join_server(&app, "lobby", "running").await;
    let request = Uuid::new_v4();
    let first = join_submit(&app, &token, &player, sleeping, request).await;
    assert_eq!(
        join_submit(&app, &token, &player, sleeping, request).await,
        first,
        "idempotent request retry"
    );
    let duplicate = join_submit(&app, &token, &player, sleeping, Uuid::new_v4()).await;
    assert_eq!(duplicate["job_id"], first["job_id"]);
    assert_eq!(duplicate["coalesced"], true);
    let job = join_lease(&app, &token).await;
    assert_eq!(job["payload"]["session_id"], player["session_id"]);
    assert_eq!(job["payload"]["native_uuid"], player["native_uuid"]);
    assert_eq!(job["payload"]["profile_id"], player["profile_id"]);
    let (status, route) = host_http(
        &app,
        &token,
        "/internal/v1/game/route",
        join_request(&player, &job, "check"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{route}");
    assert_eq!(route["ready"], false);
    let wake_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM jobs WHERE kind='server.start' AND server_id=$1")
            .bind(sleeping)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(
        wake_count, 1,
        "polling and duplicate clicks coalesce startup too"
    );
    let next = join_submit(&app, &token, &player, lobby, Uuid::new_v4()).await;
    let (status, route) = host_http(
        &app,
        &token,
        "/internal/v1/game/route",
        join_request(&player, &job, "connect"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{route}");
    assert_eq!(
        route["state"], "cancelled",
        "an old leased job cannot connect after supersession"
    );
    assert_eq!(route["ready"], false);
    let next_job = join_lease(&app, &token).await;
    assert_eq!(next_job["id"], next["job_id"]);
    let (status, _) = host_http(
        &app,
        &token,
        "/internal/v1/game/route",
        join_request(&player, &next_job, "complete"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "acceptance is not arrival");
    let (status, _) = host_http(
        &app,
        &token,
        "/internal/v1/game/route",
        join_request(&player, &next_job, "connect"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = host_http(&app, &token, "/internal/v1/game/command", json!({
        "account_id":player["account_id"],"session_id":player["session_id"],"request_id":Uuid::new_v4(),
        "command":{"type":"server_join","id":sleeping}})).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a new destination cannot race an in-flight effect"
    );
    let (status, _) = host_http(&app, &token, "/internal/v1/game/route", json!({"account_id":player["account_id"],"session_id":player["session_id"],"join_phase":"cancel"})).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = host_http(&app, &token, "/internal/v1/game/heartbeat", json!({"account_id":player["account_id"],"session_id":player["session_id"],"server_id":lobby})).await;
    assert_eq!(status, StatusCode::OK);
    let complete = join_request(&player, &next_job, "complete");
    let (status, response) =
        host_http(&app, &token, "/internal/v1/game/route", complete.clone()).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    assert_eq!(response["state"], "succeeded");
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", complete)
            .await
            .1["state"],
        "succeeded",
        "lost-response retry"
    );
    let notifications: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notifications WHERE account_id=$1 AND kind='job_finished' AND body->>'id'=$2",
    )
    .bind(id(&player, "account_id"))
    .bind(id(&next_job, "id").to_string())
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(notifications, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_reconnect_cancel_deadline_access_and_lease_security(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let native = Uuid::new_v4();
    let player = join_connect(&app, &token, native, "java").await;
    let server = join_server(&app, "official", "stopped").await;
    join_submit(&app, &token, &player, server, Uuid::new_v4()).await;
    let job = join_lease(&app, &token).await;
    let mut forged = join_request(&player, &job, "connect");
    forged["lease_token"] = json!(Uuid::new_v4());
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", forged)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let other_proxy = join_proxy(&app).await;
    assert_eq!(
        host_http(
            &app,
            &other_proxy,
            "/internal/v1/game/route",
            join_request(&player, &job, "connect")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        internal(
            &app,
            "official",
            Some(server),
            "/internal/v1/game/route",
            join_request(&player, &job, "connect")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    host_http(
        &app,
        &token,
        "/internal/v1/game/disconnect",
        json!({"account_id":player["account_id"],"session_id":player["session_id"]}),
    )
    .await;
    let again = join_connect(&app, &token, native, "java").await;
    assert_ne!(again["session_id"], player["session_id"]);
    let old = host_http(
        &app,
        &token,
        "/internal/v1/game/route",
        join_request(&player, &job, "connect"),
    )
    .await;
    assert_eq!(old.0, StatusCode::OK);
    assert_eq!(old.1["state"], "failed");
    assert_eq!(
        old.1["ready"], false,
        "disconnect invalidates the original job immediately"
    );
    let mut adopt = join_request(&again, &job, "connect");
    adopt["session_id"] = again["session_id"].clone();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", adopt)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    // A stale adapter command must not create a job for the new session.
    let (status, _) = host_http(&app,&token,"/internal/v1/game/command",json!({"account_id":player["account_id"],"session_id":player["session_id"],"request_id":Uuid::new_v4(),"command":{"type":"server_join","id":server}})).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // Exercise the admission/execute race: an adapter was authorized just before
    // reconnect, but commands must bind its original session under the account lock.
    let stale_actor = Actor {
        id: id(&again, "account_id"),
        admin: false,
        csrf: String::new(),
        session_hash: format!("game:{}", id(&player, "session_id")),
    };
    let error = commands::execute(
        &app,
        &stale_actor,
        Request {
            request_id: Uuid::new_v4(),
            command: Command::ServerJoin { id: server },
        },
    )
    .await
    .unwrap_err();
    assert_eq!(error.status, StatusCode::CONFLICT);
    let next = join_submit(&app, &token, &again, server, Uuid::new_v4()).await;
    let next_job = join_lease(&app, &token).await;
    assert_eq!(next_job["id"], next["job_id"]);
    let (status, result) = host_http(&app,&token,"/internal/v1/game/route",json!({"account_id":again["account_id"],"session_id":again["session_id"],"join_phase":"cancel"})).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["cancelled"], 1);
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&again, &next_job, "connect")
        )
        .await
        .1["state"],
        "cancelled"
    );
    join_submit(&app, &token, &again, server, Uuid::new_v4()).await;
    let expiring = join_lease(&app, &token).await;
    sqlx::query("UPDATE jobs SET created_at=now()-interval '601 seconds' WHERE id=$1")
        .bind(id(&expiring, "id"))
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&again, &expiring, "connect")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let mut failure = join_request(&again, &expiring, "fail");
    failure["error"] = json!("Startup timed out; choose the server again.");
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", failure)
            .await
            .1["state"],
        "failed"
    );
    // Admission checks occur before waking and are repeated at the effect boundary.
    sqlx::query("UPDATE servers SET visibility='private' WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    let (status, _) = host_http(&app,&token,"/internal/v1/game/command",json!({"account_id":again["account_id"],"session_id":again["session_id"],"request_id":Uuid::new_v4(),"command":{"type":"server_join","id":server}})).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    sqlx::query("UPDATE servers SET visibility='public',capabilities='{}' WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    let (status, _) = host_http(&app,&token,"/internal/v1/game/command",json!({"account_id":again["account_id"],"session_id":again["session_id"],"request_id":Uuid::new_v4(),"command":{"type":"server_join","id":server}})).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_boundary_rechecks_combat_maintenance_client_and_profile(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let player = join_connect(&app, &token, Uuid::from_u128(1234567), "bedrock").await;
    let server = join_server(&app, "official", "running").await;
    join_submit(&app, &token, &player, server, Uuid::new_v4()).await;
    let job = join_lease(&app, &token).await;
    let route = join_request(&player, &job, "connect");
    sqlx::query("UPDATE accounts SET combat_until=now()+interval '30 seconds' WHERE id=$1")
        .bind(id(&player, "account_id"))
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", route.clone())
            .await
            .0,
        StatusCode::CONFLICT
    );
    sqlx::query("UPDATE accounts SET combat_until=NULL WHERE id=$1")
        .bind(id(&player, "account_id"))
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("UPDATE servers SET maintenance=true WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", route.clone())
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    sqlx::query("UPDATE servers SET maintenance=false,capabilities=$2 WHERE id=$1")
        .bind(server)
        .bind(json!({"proxy_join":true,"bedrock":false}))
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", route.clone())
            .await
            .0,
        StatusCode::CONFLICT
    );
    sqlx::query("UPDATE servers SET capabilities=$2,last_observed_at=now()-interval '31 seconds' WHERE id=$1").bind(server).bind(json!({"proxy_join":true,"bedrock":true})).execute(&app.db).await.unwrap();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", route.clone())
            .await
            .0,
        StatusCode::CONFLICT,
        "stale running is not readiness"
    );
    sqlx::query("UPDATE servers SET last_observed_at=now() WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    let mut mismatched = job.clone();
    mismatched["payload"]["profile_id"] = json!(Uuid::new_v4());
    sqlx::query("UPDATE jobs SET payload=$2 WHERE id=$1")
        .bind(id(&job, "id"))
        .bind(&mismatched["payload"])
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", route.clone())
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    sqlx::query("UPDATE jobs SET payload=$2 WHERE id=$1")
        .bind(id(&job, "id"))
        .bind(&job["payload"])
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET banned_until=now()+interval '1 hour' WHERE id=$1")
        .bind(id(&player, "account_id"))
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", route)
            .await
            .0,
        StatusCode::CONFLICT
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_waiting_queue_rotates_without_starving_other_players(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let a = join_connect(&app, &token, Uuid::new_v4(), "java").await;
    let b = join_connect(&app, &token, Uuid::new_v4(), "java").await;
    let server = join_server(&app, "official", "stopped").await;
    let first = join_submit(&app, &token, &a, server, Uuid::new_v4()).await;
    let second = join_submit(&app, &token, &b, server, Uuid::new_v4()).await;
    let first_job = join_lease(&app, &token).await;
    assert_eq!(first_job["id"], first["job_id"]);
    let (status, _) = host_http(&app,&token,&format!("/internal/v1/jobs/{}/ack",id(&first_job,"id")),json!({"lease_token":first_job["lease_token"],"state":"waiting","progress":{"phase":"waking"}})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(join_lease(&app, &token).await["id"], second["job_id"]);
    sqlx::query("UPDATE jobs SET updated_at=now()-interval '6 seconds' WHERE id=$1")
        .bind(id(&first_job, "id"))
        .execute(&app.db)
        .await
        .unwrap();
    let retry = join_lease(&app, &token).await;
    assert_eq!(retry["id"], first["job_id"]);
    assert_ne!(retry["lease_token"], first_job["lease_token"]);
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&a, &first_job, "connect")
        )
        .await
        .0,
        StatusCode::FORBIDDEN,
        "an old lease cannot perform an effect"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_disconnect_after_observed_arrival_keeps_success(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let player = join_connect(&app, &token, Uuid::new_v4(), "java").await;
    let target = join_server(&app, "official", "running").await;
    let travel = join_submit(&app, &token, &player, target, Uuid::new_v4()).await;
    let job = join_lease(&app, &token).await;
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&player, &job, "connect")
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(host_http(&app,&token,"/internal/v1/game/heartbeat",json!({"account_id":player["account_id"],"session_id":player["session_id"],"server_id":target})).await.0,StatusCode::OK);
    // The client leaves after observation but before the success response arrives.
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/disconnect",
            json!({"account_id":player["account_id"],"session_id":player["session_id"]})
        )
        .await
        .0,
        StatusCode::OK
    );
    let stored: Value = sqlx::query_scalar("SELECT to_jsonb(jobs) FROM jobs WHERE id=$1")
        .bind(id(&travel, "job_id"))
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(stored["state"], "succeeded");
    assert_eq!(stored["result"]["effect"], "committed");
    assert_eq!(stored["result"]["session_id"], player["session_id"]);
    assert_eq!(stored["progress"]["phase"], "arrived");
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&player, &job, "complete")
        )
        .await
        .1["state"],
        "succeeded",
        "a retry cannot turn observed arrival into a disconnected failure"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_lost_arrival_ack_recovers_observation_after_lobby_fallback(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let player = join_connect(&app, &token, Uuid::new_v4(), "java").await;
    let target = join_server(&app, "official", "running").await;
    let lobby = join_server(&app, "lobby", "running").await;
    join_submit(&app, &token, &player, target, Uuid::new_v4()).await;
    let job = join_lease(&app, &token).await;
    host_http(
        &app,
        &token,
        "/internal/v1/game/route",
        join_request(&player, &job, "connect"),
    )
    .await;
    for server in [target, lobby] {
        assert_eq!(host_http(&app,&token,"/internal/v1/game/heartbeat",json!({"account_id":player["account_id"],"session_id":player["session_id"],"server_id":server})).await.0,StatusCode::OK);
    }
    let (status, result) = host_http(
        &app,
        &token,
        "/internal/v1/game/route",
        join_request(&player, &job, "complete"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(
        result["state"], "succeeded",
        "arrival was observed before a later recovery"
    );
    let result: Value = sqlx::query_scalar("SELECT result FROM jobs WHERE id=$1")
        .bind(id(&job, "id"))
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(result["server_id"], json!(target));
    assert_eq!(result["actual_server_id"], json!(lobby));
}

// Requires the coordinator's shared ACK guard documented in .local/ux/result.md.
#[sqlx::test(migrations = "../../migrations")]
async fn join_generic_ack_cannot_manufacture_arrival(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let player = join_connect(&app, &token, Uuid::new_v4(), "java").await;
    let target = join_server(&app, "official", "running").await;
    join_submit(&app, &token, &player, target, Uuid::new_v4()).await;
    let job = join_lease(&app, &token).await;
    let ack = format!("/internal/v1/jobs/{}/ack", id(&job, "id"));
    for state in ["succeeded", "failed"] {
        let (status, _) = host_http(&app, &token, &ack, json!({"lease_token":job["lease_token"],"state":state,"result":{"effect":if state == "succeeded" {"committed"} else {"none"}}})).await;
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "bound terminal ACK must use route reconciliation"
        );
    }
    assert_eq!(host_http(&app, &token, &ack, json!({"lease_token":job["lease_token"],"state":"leased","progress":{"phase":"preparing"}})).await.0, StatusCode::OK);
    assert_eq!(host_http(&app,&token,"/internal/v1/game/heartbeat",json!({"account_id":player["account_id"],"session_id":player["session_id"],"server_id":target})).await.0,StatusCode::OK);
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&player, &job, "complete")
        )
        .await
        .1["state"],
        "succeeded"
    );
    let result: Value = sqlx::query_scalar("SELECT result FROM jobs WHERE id=$1")
        .bind(id(&job, "id"))
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(
            &app,
            &token,
            &ack,
            json!({"lease_token":job["lease_token"],"state":"succeeded","result":result})
        )
        .await
        .0,
        StatusCode::OK,
        "terminal idempotency precedes generic guard"
    );
    // Legacy, unbound work may fail safely; it cannot connect or claim success.
    let legacy = Uuid::new_v4();
    sqlx::query("INSERT INTO jobs(id,actor,server_id,worker,kind,payload) VALUES($1,$2,$3,'proxy','player.join','{}')")
        .bind(legacy).bind(id(&player,"account_id")).bind(target).execute(&app.db).await.unwrap();
    let old = join_lease(&app, &token).await;
    assert_eq!(old["id"], json!(legacy));
    let ack = format!("/internal/v1/jobs/{legacy}/ack");
    assert_eq!(host_http(&app,&token,&ack,json!({"lease_token":old["lease_token"],"state":"succeeded","result":{"effect":"committed"}})).await.0,StatusCode::CONFLICT);
    assert_eq!(
        host_http(
            &app,
            &token,
            &ack,
            json!({"lease_token":old["lease_token"],"state":"failed","result":{"effect":"none"}})
        )
        .await
        .0,
        StatusCode::OK
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_failure_and_disconnect_reconcile_arrival_history(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let target = join_server(&app, "official", "running").await;
    let lobby = join_server(&app, "lobby", "running").await;
    for (fallback, disconnect) in [(false, false), (true, false), (true, true)] {
        let player = join_connect(&app, &token, Uuid::new_v4(), "java").await;
        join_submit(&app, &token, &player, target, Uuid::new_v4()).await;
        let job = join_lease(&app, &token).await;
        assert_eq!(
            host_http(
                &app,
                &token,
                "/internal/v1/game/route",
                join_request(&player, &job, "connect")
            )
            .await
            .0,
            StatusCode::OK
        );
        for server in if fallback {
            vec![target, lobby]
        } else {
            vec![target]
        } {
            assert_eq!(host_http(&app,&token,"/internal/v1/game/heartbeat",json!({"account_id":player["account_id"],"session_id":player["session_id"],"server_id":server})).await.0,StatusCode::OK);
        }
        let (path, body) = if disconnect {
            (
                "/internal/v1/game/disconnect",
                json!({"account_id":player["account_id"],"session_id":player["session_id"]}),
            )
        } else {
            (
                "/internal/v1/game/route",
                join_request(&player, &job, "fail"),
            )
        };
        assert_eq!(host_http(&app, &token, path, body).await.0, StatusCode::OK);
        let stored: Value = sqlx::query_scalar("SELECT to_jsonb(jobs) FROM jobs WHERE id=$1")
            .bind(id(&job, "id"))
            .fetch_one(&app.db)
            .await
            .unwrap();
        assert_eq!(stored["state"], "succeeded");
        assert_eq!(stored["result"]["effect"], "committed");
        assert_eq!(
            stored["result"]["actual_server_id"],
            json!(if fallback { lobby } else { target })
        );
        assert!(stored["error"].is_null());
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_history_rejects_wrong_identity_old_observation_and_current_mismatch(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let player = join_connect(&app, &token, Uuid::new_v4(), "java").await;
    let target = join_server(&app, "official", "running").await;
    let lobby = join_server(&app, "lobby", "running").await;
    join_submit(&app, &token, &player, target, Uuid::new_v4()).await;
    let job = join_lease(&app, &token).await;
    for server in [target, lobby] {
        assert_eq!(host_http(&app,&token,"/internal/v1/game/heartbeat",json!({"account_id":player["account_id"],"session_id":player["session_id"],"server_id":server})).await.0,StatusCode::OK);
    }
    let complete = join_request(&player, &job, "complete");
    for field in ["native_uuid", "profile_id", "session_id"] {
        let mut payload = job["payload"].clone();
        payload[field] = json!(Uuid::new_v4());
        sqlx::query("UPDATE jobs SET payload=$2 WHERE id=$1")
            .bind(id(&job, "id"))
            .bind(payload)
            .execute(&app.db)
            .await
            .unwrap();
        assert_ne!(
            host_http(&app, &token, "/internal/v1/game/route", complete.clone())
                .await
                .0,
            StatusCode::OK,
            "wrong {field}"
        );
    }
    sqlx::query("UPDATE jobs SET payload=$2 WHERE id=$1")
        .bind(id(&job, "id"))
        .bind(&job["payload"])
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("UPDATE game_session_history SET started_at=now()-interval '2 minutes',last_seen_at=now()-interval '1 minute' WHERE session_id=$1 AND server_id=$2").bind(id(&player,"session_id")).bind(target).execute(&app.db).await.unwrap();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", complete.clone())
            .await
            .0,
        StatusCode::CONFLICT,
        "history predates request"
    );
    sqlx::query(
        "UPDATE game_session_history SET last_seen_at=now() WHERE session_id=$1 AND server_id=$2",
    )
    .bind(id(&player, "session_id"))
    .bind(target)
    .execute(&app.db)
    .await
    .unwrap();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", complete.clone())
            .await
            .0,
        StatusCode::CONFLICT,
        "target observation newer than current location"
    );
    sqlx::query("DELETE FROM game_session_history WHERE session_id=$1 AND server_id=$2")
        .bind(id(&player, "session_id"))
        .bind(lobby)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", complete)
            .await
            .0,
        StatusCode::CONFLICT,
        "current location has no corroborating observation"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_expired_connection_requires_proxy_fence_before_replacement(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let player = join_connect(&app, &token, Uuid::new_v4(), "java").await;
    let target = join_server(&app, "official", "running").await;
    let lobby = join_server(&app, "lobby", "running").await;
    join_submit(&app, &token, &player, target, Uuid::new_v4()).await;
    let job = join_lease(&app, &token).await;
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&player, &job, "connect")
        )
        .await
        .0,
        StatusCode::OK
    );
    sqlx::query("UPDATE jobs SET lease_until=now()-interval '1 second' WHERE id=$1")
        .bind(id(&job, "id"))
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE game_sessions SET route_expires_at=now()-interval '1 second' WHERE account_id=$1",
    )
    .bind(id(&player, "account_id"))
    .execute(&app.db)
    .await
    .unwrap();
    let cancel = json!({"account_id":player["account_id"],"session_id":player["session_id"],"join_phase":"cancel"});
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", cancel.clone())
            .await
            .0,
        StatusCode::CONFLICT,
        "time alone cannot fence a socket"
    );
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&player, &job, "fail")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    // The trusted proxy sends this only after natural completion of its old future.
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&player, &job, "fence")
        )
        .await
        .1["state"],
        "failed"
    );
    let next = join_submit(&app, &token, &player, lobby, Uuid::new_v4()).await;
    assert_ne!(next["job_id"], job["id"]);
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&player, &job, "connect")
        )
        .await
        .1["state"],
        "failed",
        "old connect stays fenced"
    );
    assert_eq!(
        host_http(&app, &token, "/internal/v1/game/route", cancel)
            .await
            .1["cancelled"],
        1
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn join_ten_minute_deadline_includes_queue_and_startup(pool: PgPool) {
    let app = app(pool);
    let token = join_proxy(&app).await;
    let player = join_connect(&app, &token, Uuid::new_v4(), "java").await;
    let target = join_server(&app, "official", "stopped").await;
    join_submit(&app, &token, &player, target, Uuid::new_v4()).await;
    let job = join_lease(&app, &token).await;
    sqlx::query("UPDATE jobs SET created_at=now()-interval '5 minutes' WHERE id=$1")
        .bind(id(&job, "id"))
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        host_http(
            &app,
            &token,
            "/internal/v1/game/route",
            join_request(&player, &job, "check")
        )
        .await
        .0,
        StatusCode::OK,
        "VM120 + Minecraft120 + queue fits"
    );
    sqlx::query(
        "UPDATE jobs SET state='waiting',created_at=now()-interval '601 seconds' WHERE id=$1",
    )
    .bind(id(&job, "id"))
    .execute(&app.db)
    .await
    .unwrap();
    let (status, result) = host_http(
        &app,
        &token,
        "/internal/v1/game/heartbeat",
        json!({"account_id":player["account_id"],"session_id":player["session_id"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result["join_results"][0]["state"], "failed");
    assert!(
        result["join_results"][0]["error"]
            .as_str()
            .unwrap()
            .contains("10 minutes")
    );
    assert!(
        host_http(&app, &token, "/internal/v1/poll", json!({}))
            .await
            .1["job"]
            .is_null(),
        "deadline cannot restart queue work"
    );
}
