use axum::{
    body::{Body, to_bytes},
    http::{Request as HttpRequest, StatusCode},
};
use lkjmc_core::{
    App,
    auth::{self, Actor},
    commands::{self, Command, Request},
    config::{Action, Config},
    economy,
};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn app(pool: PgPool) -> App {
    App {
        db: pool,
        config: Arc::new(Config {
            database_url: String::new(),
            bind: "127.0.0.1:18091".parse().unwrap(),
            public_url: "http://127.0.0.1:18091".into(),
            storage: std::env::temp_dir().join(format!("lkjmc-test-{}", Uuid::new_v4())),
            web: "web/dist".into(),
            oidc_issuer: None,
            oidc_client_id: None,
            oidc_secret: None,
            voice_url: None,
            voice_key: None,
            voice_secret: None,
            development: true,
            action: Action::Migrate,
        }),
        http: reqwest::Client::new(),
    }
}
async fn account(app: &App, name: &str, admin: bool) -> Actor {
    let mut tx = app.db.begin().await.unwrap();
    let id = auth::create_account(&mut tx, name).await.unwrap();
    sqlx::query("UPDATE accounts SET administrator=$2 WHERE id=$1")
        .bind(id)
        .bind(admin)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    Actor {
        id,
        admin,
        csrf: String::new(),
        session_hash: String::new(),
    }
}
async fn run(app: &App, actor: &Actor, command: Command) -> Value {
    commands::execute(
        app,
        actor,
        Request {
            request_id: Uuid::new_v4(),
            command,
        },
    )
    .await
    .unwrap()["result"]
        .clone()
}
fn id(v: &Value, key: &str) -> Uuid {
    Uuid::parse_str(
        v[key]
            .as_str()
            .unwrap_or_else(|| panic!("missing {key} in {v}")),
    )
    .unwrap()
}
async fn fund(app: &App, actor: &Actor, coins: i64) {
    let mut tx = app.db.begin().await.unwrap();
    economy::book(
        &mut tx,
        actor.id,
        &format!("fixture:{}", Uuid::new_v4()),
        "fixture",
        json!({}),
        &[(actor.id, coins)],
        true,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
}
async fn http(
    app: &App,
    actor: &Actor,
    method: &str,
    path: &str,
    body: Value,
    csrf: bool,
) -> (StatusCode, Value) {
    let mut tx = app.db.begin().await.unwrap();
    let (token, secret) = auth::new_session(&mut tx, actor.id).await.unwrap();
    tx.commit().await.unwrap();
    let mut req = HttpRequest::builder()
        .method(method)
        .uri(path)
        .header("cookie", format!("lkjmc_session={token}"))
        .header("content-type", "application/json");
    if csrf {
        req = req
            .header("x-csrf-token", secret)
            .header("origin", "http://127.0.0.1:18091");
    }
    let response = lkjmc_core::router(app.clone())
        .oneshot(
            req.body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| json!({"raw":String::from_utf8_lossy(&bytes)})),
    )
}
async fn internal(
    app: &App,
    role: &str,
    server: Option<Uuid>,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let token = auth::random_token();
    sqlx::query("INSERT INTO service_credentials(id,name,token_hash,role,server_id) VALUES($1,'test',$2,$3,$4)").bind(Uuid::new_v4()).bind(auth::hash(&token)).bind(role).bind(server).execute(&app.db).await.unwrap();
    let req = HttpRequest::builder()
        .method("POST")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = lkjmc_core::router(app.clone()).oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}
async fn official(app: &App) -> (Uuid, Uuid) {
    let server = Uuid::new_v4();
    let world = Uuid::new_v4();
    sqlx::query("INSERT INTO servers(id,name,kind,visibility,desired,observed,version,software,memory_mib,cpu_millis,storage_mib,last_observed_at) VALUES($1,'SMP','official','public','running','running','test','paper',2048,2000,10240,now())").bind(server).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO worlds(id,server_id,name,kind) VALUES($1,$2,'living','living')")
        .bind(world)
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    (server, world)
}

async fn host_token(app: &App) -> String {
    let token = auth::random_token();
    sqlx::query("INSERT INTO service_credentials(id,name,token_hash,role) VALUES($1,'host-fixture',$2,'host')").bind(Uuid::new_v4()).bind(auth::hash(&token)).execute(&app.db).await.unwrap();
    token
}
async fn host_http(app: &App, token: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let response = lkjmc_core::router(app.clone())
        .oneshot(
            HttpRequest::builder()
                .method("POST")
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}
async fn custom_server(app: &App, owner: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO servers(id,name,owner,kind,visibility,desired,observed,version,software,memory_mib,cpu_millis,storage_mib,last_observed_at) VALUES($1,'tenant-fixture',$2,'custom','public','running','running','test','paper',2048,1000,10240,now())").bind(id).bind(owner).execute(&app.db).await.unwrap();
    id
}

#[sqlx::test(migrations = "../../migrations")]
async fn host_rechecks_revoked_permission_before_authorizing_effects(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "所有者", false).await;
    let member = account(&app, "共同管理者", false).await;
    let server = custom_server(&app, owner.id).await;
    sqlx::query(
        "INSERT INTO server_members(server_id,account_id,role) VALUES($1,$2,'administrator')",
    )
    .bind(server)
    .bind(member.id)
    .execute(&app.db)
    .await
    .unwrap();
    let queued = run(
        &app,
        &member,
        Command::ServerConsole {
            id: server,
            line: "say queued".into(),
        },
    )
    .await;
    sqlx::query("DELETE FROM server_members WHERE server_id=$1 AND account_id=$2")
        .bind(server)
        .bind(member.id)
        .execute(&app.db)
        .await
        .unwrap();
    let token = host_token(&app).await;
    let (status, response) = host_http(&app, &token, "/internal/v1/poll", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    let job = &response["job"];
    assert_eq!(job["id"], queued["job_id"]);
    let path = format!("/internal/v1/jobs/{}/context", id(job, "id"));
    assert_eq!(
        host_http(&app, &token, &path, json!({"lease_token":Uuid::new_v4()}))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let (status, context) = host_http(
        &app,
        &token,
        &path,
        json!({"lease_token":job["lease_token"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(context["rejected"].is_string());
    let authorized: bool =
        sqlx::query_scalar("SELECT host_authorized_at IS NOT NULL FROM jobs WHERE id=$1")
            .bind(id(job, "id"))
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert!(!authorized);
    let path = format!("/internal/v1/jobs/{}/ack", id(job, "id"));
    let (status,result)=host_http(&app,&token,&path,json!({"lease_token":job["lease_token"],"state":"failed","result":{"effect":"none"},"error":"権限が変更されました。"})).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let mut db = app.db.acquire().await.unwrap();
    lkjmc_core::hosting::can_join(&mut db, owner.id, server)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn host_recovery_is_fenced_and_maintenance_preserves_existing_sessions(pool: PgPool) {
    let app = app(pool);
    let owner = account(&app, "サーバー所有者", false).await;
    let visitor = account(&app, "参加者", false).await;
    let server = custom_server(&app, owner.id).await;
    let queued = run(&app, &owner, Command::ServerBackup { id: server }).await;
    let first = host_token(&app).await;
    let second = host_token(&app).await;
    let (_, response) = host_http(&app, &first, "/internal/v1/poll", json!({})).await;
    let job = response["job"].clone();
    assert_eq!(job["id"], queued["job_id"]);
    let context_path = format!("/internal/v1/jobs/{}/context", id(&job, "id"));
    let ack_path = format!("/internal/v1/jobs/{}/ack", id(&job, "id"));
    assert_eq!(
        host_http(
            &app,
            &first,
            &context_path,
            json!({"lease_token":job["lease_token"]})
        )
        .await
        .0,
        StatusCode::OK
    );
    let mut db = app.db.acquire().await.unwrap();
    assert!(
        lkjmc_core::hosting::can_join(&mut db, visitor.id, server)
            .await
            .is_err()
    );
    lkjmc_core::hosting::can_remain(&mut db, visitor.id, server)
        .await
        .unwrap();
    drop(db);
    // Queue a stop during backup. It must remain pending until backup recovery ends.
    let stop = run(&app, &owner, Command::ServerStop { id: server }).await;
    assert!(
        host_http(&app, &second, "/internal/v1/poll", json!({}))
            .await
            .1["job"]
            .is_null()
    );
    assert_eq!(
        host_http(
            &app,
            &first,
            &ack_path,
            json!({"lease_token":job["lease_token"],"state":"waiting"})
        )
        .await
        .0,
        StatusCode::OK
    );
    sqlx::query("UPDATE jobs SET updated_at=now()-interval '10 seconds' WHERE id=$1")
        .bind(id(&job, "id"))
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET banned_until=now()+interval '1 day' WHERE id=$1")
        .bind(owner.id)
        .execute(&app.db)
        .await
        .unwrap();
    let (_, response) = host_http(&app, &second, "/internal/v1/poll", json!({})).await;
    let recovered = response["job"].clone();
    assert_eq!(recovered["id"], job["id"]);
    assert_ne!(recovered["lease_token"], job["lease_token"]);
    assert_eq!(
        host_http(
            &app,
            &first,
            &context_path,
            json!({"lease_token":job["lease_token"]})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (status, context) = host_http(
        &app,
        &second,
        &context_path,
        json!({"lease_token":recovered["lease_token"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(context["rejected"].is_null(), "{context}");
    // Database protocol fixture only: this does not assert that an actual VM was exported.
    let (status,result)=host_http(&app,&second,&ack_path,json!({"lease_token":recovered["lease_token"],"state":"succeeded","result":{"effect":"committed","verified":true,"fixture":true}})).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let row = sqlx::query("SELECT maintenance,desired FROM servers WHERE id=$1")
        .bind(server)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert!(row.get::<bool, _>("maintenance"));
    assert_eq!(row.get::<String, _>("desired"), "stopped");
    sqlx::query("UPDATE accounts SET banned_until=NULL WHERE id=$1")
        .bind(owner.id)
        .execute(&app.db)
        .await
        .unwrap();
    let (_, response) = host_http(&app, &second, "/internal/v1/poll", json!({})).await;
    let job = response["job"].clone();
    assert_eq!(job["id"], stop["job_id"]);
    let path = format!("/internal/v1/jobs/{}/context", id(&job, "id"));
    assert_eq!(
        host_http(
            &app,
            &second,
            &path,
            json!({"lease_token":job["lease_token"]})
        )
        .await
        .0,
        StatusCode::OK
    );
    let path = format!("/internal/v1/jobs/{}/ack", id(&job, "id"));
    assert_eq!(host_http(&app,&second,&path,json!({"lease_token":job["lease_token"],"state":"failed","result":{"effect":"uncertain"}})).await.0,StatusCode::CONFLICT);
    let (status,result)=host_http(&app,&second,&path,json!({"lease_token":job["lease_token"],"state":"succeeded","result":{"effect":"committed","observed":"stopped"}})).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let mut db = app.db.acquire().await.unwrap();
    lkjmc_core::hosting::can_join(&mut db, visitor.id, server)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn proxy_routes_recheck_identity_membership_and_client_capability(pool: PgPool) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    sqlx::query("UPDATE servers SET capabilities='{\"proxy_join\":true,\"bedrock\":false}',visibility='private' WHERE id=$1").bind(server).execute(&app.db).await.unwrap();
    let native = Uuid::new_v4();
    let session = Uuid::new_v4();
    let connect = json!({"issuer":"java","subject":native,"native_uuid":native,"session_id":session,"display_name":"VerifiedJava"});
    let (status, user) = internal(
        &app,
        "proxy",
        None,
        "/internal/v1/game/connect",
        connect.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{user}");
    let actor = id(&user, "account_id");
    let mut duplicate = connect;
    duplicate["session_id"] = json!(Uuid::new_v4());
    assert_eq!(
        internal(&app, "proxy", None, "/internal/v1/game/connect", duplicate)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let request = json!({"account_id":actor,"session_id":session,"server_id":server});
    assert_eq!(
        internal(
            &app,
            "proxy",
            None,
            "/internal/v1/game/route",
            request.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    sqlx::query("INSERT INTO server_members VALUES($1,$2,'guest')")
        .bind(server)
        .bind(actor)
        .execute(&app.db)
        .await
        .unwrap();
    let (status, route) = internal(
        &app,
        "proxy",
        None,
        "/internal/v1/game/route",
        request.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{route}");
    assert_eq!(route["ready"], true);
    let pending: Option<Uuid> =
        sqlx::query_scalar("SELECT pending_server_id FROM game_sessions WHERE account_id=$1")
            .bind(actor)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(pending, Some(server));
    sqlx::query("DELETE FROM server_members WHERE account_id=$1")
        .bind(actor)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        internal(
            &app,
            "proxy",
            None,
            "/internal/v1/game/route",
            request.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        internal(&app, "proxy", None, "/internal/v1/game/heartbeat", request)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    sqlx::query("UPDATE servers SET visibility='public' WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    let bedrock = Uuid::from_u128(2535412345678901);
    let bs = Uuid::new_v4();
    let bc = json!({"issuer":"bedrock","subject":"2535412345678901","native_uuid":bedrock,"session_id":bs,"display_name":"VerifiedBedrock"});
    let mut wrong = bc.clone();
    wrong["native_uuid"] = json!(native);
    assert_eq!(
        internal(&app, "proxy", None, "/internal/v1/game/connect", wrong)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let (status, bedrock_user) =
        internal(&app, "proxy", None, "/internal/v1/game/connect", bc).await;
    assert_eq!(status, StatusCode::OK, "{bedrock_user}");
    let request =
        json!({"account_id":bedrock_user["account_id"],"session_id":bs,"server_id":server});
    assert_eq!(
        internal(
            &app,
            "proxy",
            None,
            "/internal/v1/game/route",
            request.clone()
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    sqlx::query("UPDATE servers SET capabilities=capabilities || '{\"bedrock\":true}' WHERE id=$1")
        .bind(server)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(
        internal(
            &app,
            "proxy",
            None,
            "/internal/v1/game/route",
            request.clone()
        )
        .await
        .0,
        StatusCode::OK
    );
    sqlx::query(
        "UPDATE game_sessions SET combat_until=now()+interval '30 seconds' WHERE account_id=$1",
    )
    .bind(id(&bedrock_user, "account_id"))
    .execute(&app.db)
    .await
    .unwrap();
    assert_eq!(
        internal(
            &app,
            "proxy",
            None,
            "/internal/v1/game/route",
            request.clone()
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        internal(
            &app,
            "official",
            Some(server),
            "/internal/v1/game/route",
            request
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}

async fn acknowledge(
    app: &App,
    server: Uuid,
    job: Uuid,
    state: &str,
    result: Value,
) -> (StatusCode, Value) {
    acknowledge_progress(app, server, job, state, result, json!({})).await
}

async fn acknowledge_progress(
    app: &App,
    server: Uuid,
    job: Uuid,
    state: &str,
    result: Value,
    progress: Value,
) -> (StatusCode, Value) {
    let token = auth::random_token();
    let credential = Uuid::new_v4();
    let lease = Uuid::new_v4();
    sqlx::query("INSERT INTO service_credentials(id,name,token_hash,role,server_id) VALUES($1,'worker-test',$2,'official',$3)")
        .bind(credential).bind(auth::hash(&token)).bind(server).execute(&app.db).await.unwrap();
    sqlx::query("UPDATE jobs SET state='leased',lease_owner=$2,lease_token=$3,lease_until=now()+interval '90 seconds' WHERE id=$1")
        .bind(job).bind(credential).bind(lease).execute(&app.db).await.unwrap();
    let response = lkjmc_core::router(app.clone())
        .oneshot(
            HttpRequest::builder()
                .method("POST")
                .uri(format!("/internal/v1/jobs/{job}/ack"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"lease_token":lease,"state":state,"result":result,"progress":progress})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[sqlx::test(migrations = "../../migrations")]
async fn pet_consent_binds_immutable_manifest_and_cancellation_precedes_removal(pool: PgPool) {
    let app = app(pool);
    let (server, world) = official(&app).await;
    let owner = account(&app, "建築者", false).await;
    let pet = account(&app, "飼い主", false).await;
    let profile: Uuid = sqlx::query_scalar("SELECT id FROM profiles WHERE account_id=$1")
        .bind(owner.id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO game_sessions(account_id,profile_id,native_uuid,session_id,server_id,lease_until) VALUES($1,$2,$3,$4,$5,now()+interval '45 seconds')").bind(owner.id).bind(profile).bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(server).execute(&app.db).await.unwrap();
    let claim = Uuid::new_v4();
    sqlx::query("INSERT INTO claims(id,owner,world_id,name,min_x,min_z,max_x,max_z,state) VALUES($1,$2,$3,'建物',100,100,100,100,'active')").bind(claim).bind(owner.id).bind(world).execute(&app.db).await.unwrap();
    let capture = run(
        &app,
        &owner,
        Command::AssetCapture {
            owner: None,
            kind: "building".into(),
            title: "同意を検証する建物".into(),
            selection: json!({"claim_id":claim}),
            include_contents: true,
        },
    )
    .await;
    let asset = id(&capture, "asset_id");
    let job = id(&capture, "job_id");
    let manifest = json!({"asset_id":asset,"version":1,"required_consents":[pet.id],"blocks":4});
    assert_eq!(
        acknowledge_progress(
            &app,
            server,
            job,
            "leased",
            json!({}),
            json!({"phase":"awaiting_consent","manifest":manifest})
        )
        .await
        .0,
        StatusCode::OK
    );
    let digest: String = sqlx::query_scalar("SELECT manifest_sha256 FROM assets WHERE id=$1")
        .bind(asset)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert!(
        commands::execute(
            &app,
            &owner,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::AssetConsent {
                    id: asset,
                    manifest_sha256: digest.clone()
                }
            }
        )
        .await
        .is_err()
    );
    assert_eq!(
        acknowledge_progress(
            &app,
            server,
            job,
            "leased",
            json!({}),
            json!({"phase":"removing","manifest":manifest})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    run(
        &app,
        &pet,
        Command::AssetConsent {
            id: asset,
            manifest_sha256: digest,
        },
    )
    .await;
    let mut altered = manifest.clone();
    altered["blocks"] = json!(5);
    assert_eq!(
        acknowledge_progress(
            &app,
            server,
            job,
            "leased",
            json!({}),
            json!({"phase":"awaiting_consent","manifest":altered})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert!(
        commands::execute(
            &app,
            &owner,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::ClaimRelease { id: claim }
            }
        )
        .await
        .is_err()
    );
    run(&app, &owner, Command::AssetWithdraw { id: asset }).await;
    assert_eq!(
        acknowledge_progress(
            &app,
            server,
            job,
            "leased",
            json!({}),
            json!({"phase":"removing","manifest":manifest})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        acknowledge(&app, server, job, "failed", json!({"effect":"none"}))
            .await
            .0,
        StatusCode::OK
    );
    let state: (String, Option<Uuid>) =
        sqlx::query_as("SELECT state,locked_claim_id FROM assets WHERE id=$1")
            .bind(asset)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(state, ("cancelled".into(), None));
    run(&app, &owner, Command::ClaimRelease { id: claim }).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn moving_accounts_and_stale_admin_permissions_cannot_mutate(pool: PgPool) {
    let app = app(pool);
    let user = account(&app, "移行中", false).await;
    let admin = account(&app, "旧管理者", true).await;
    sqlx::query("UPDATE profiles SET status='moving' WHERE account_id=$1")
        .bind(user.id)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET administrator=false WHERE id=$1")
        .bind(admin.id)
        .execute(&app.db)
        .await
        .unwrap();
    for (actor, command) in [
        (
            &user,
            Command::RoomCreate {
                name: "凍結確認".into(),
            },
        ),
        (
            &admin,
            Command::RankSet {
                target: user.id,
                rank: 0,
            },
        ),
    ] {
        assert!(
            commands::execute(
                &app,
                actor,
                Request {
                    request_id: Uuid::new_v4(),
                    command
                }
            )
            .await
            .is_err()
        );
    }
    fund(&app, &admin, 500).await;
    assert!(
        commands::execute(
            &app,
            &admin,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::WalletTransfer {
                    owner: None,
                    target: user.id,
                    amount: 100
                }
            }
        )
        .await
        .is_err()
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn official_event_replay_is_bound_to_verified_server_session(pool: PgPool) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    let native = Uuid::new_v4();
    let session = Uuid::new_v4();
    let (status,connected)=internal(&app,"proxy",None,"/internal/v1/game/connect",json!({"issuer":"java","subject":native,"native_uuid":native,"session_id":session,"display_name":"event probe"})).await;
    assert_eq!(status, StatusCode::OK, "{connected}");
    let account = id(&connected, "account_id");
    let heartbeat = json!({"account_id":account,"session_id":session,"server_id":server});
    assert_eq!(
        internal(
            &app,
            "proxy",
            None,
            "/internal/v1/game/heartbeat",
            heartbeat.clone()
        )
        .await
        .0,
        StatusCode::OK
    );
    let event = json!({"id":Uuid::new_v4(),"account_id":account,"session_id":session,"occurred_at":chrono::Utc::now(),"kind":"block.placed","payload":{"amount":1000}});
    assert_eq!(
        internal(
            &app,
            "proxy",
            None,
            "/internal/v1/game/disconnect",
            heartbeat
        )
        .await
        .0,
        StatusCode::OK
    );
    let (status, body) = internal(
        &app,
        "official",
        Some(server),
        "/internal/v1/game/event",
        event.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let before:Value=sqlx::query_scalar("SELECT jsonb_agg(to_jsonb(p) ORDER BY achievement) FROM achievement_progress p WHERE owner=$1").bind(account).fetch_one(&app.db).await.unwrap();
    assert!(before.as_array().is_some_and(|a| !a.is_empty()));
    let (status, body) = internal(
        &app,
        "official",
        Some(server),
        "/internal/v1/game/event",
        event.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["duplicate"], true);
    let after:Value=sqlx::query_scalar("SELECT jsonb_agg(to_jsonb(p) ORDER BY achievement) FROM achievement_progress p WHERE owner=$1").bind(account).fetch_one(&app.db).await.unwrap();
    assert_eq!(before, after);
    let mut forged = event.clone();
    forged["id"] = json!(Uuid::new_v4());
    forged["session_id"] = json!(Uuid::new_v4());
    assert_eq!(
        internal(
            &app,
            "official",
            Some(server),
            "/internal/v1/game/event",
            forged
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        internal(&app, "host", None, "/internal/v1/game/event", event)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn land_escrow_is_unique_and_can_be_withdrawn_before_releasing_claim(pool: PgPool) {
    let app = app(pool);
    let (server, world) = official(&app).await;
    let owner = account(&app, "土地所有者", false).await;
    let claim = Uuid::new_v4();
    let asset = Uuid::new_v4();
    sqlx::query("INSERT INTO claims(id,owner,world_id,name,min_x,min_z,max_x,max_z,state) VALUES($1,$2,$3,'保管テスト',100,100,100,100,'active')").bind(claim).bind(owner.id).bind(world).execute(&app.db).await.unwrap();
    sqlx::query("INSERT INTO assets(id,owner,kind,title,state,claim_id,locked_claim_id) VALUES($1,$2,'land','土地','escrowed',$3,$3)").bind(asset).bind(owner.id).bind(claim).execute(&app.db).await.unwrap();
    let duplicate=sqlx::query("INSERT INTO assets(id,owner,kind,title,state,claim_id,locked_claim_id) VALUES($1,$2,'land','重複','capturing',$3,$3)").bind(Uuid::new_v4()).bind(owner.id).bind(claim).execute(&app.db).await.unwrap_err();
    assert_eq!(
        duplicate.as_database_error().unwrap().code().as_deref(),
        Some("23505")
    );
    assert!(
        commands::execute(
            &app,
            &owner,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::ClaimRelease { id: claim }
            }
        )
        .await
        .is_err()
    );
    let other = account(&app, "別人", false).await;
    assert!(
        commands::execute(
            &app,
            &other,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::AssetWithdraw { id: asset }
            }
        )
        .await
        .is_err()
    );
    run(&app, &owner, Command::AssetWithdraw { id: asset }).await;
    let release = run(&app, &owner, Command::ClaimRelease { id: claim }).await;
    assert_eq!(
        acknowledge(
            &app,
            server,
            id(&release, "job_id"),
            "succeeded",
            json!({"effect":"committed"})
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT state FROM claims WHERE id=$1")
            .bind(claim)
            .fetch_one(&app.db)
            .await
            .unwrap(),
        "released"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn queued_home_jobs_reserve_slots_until_success_or_failure(pool: PgPool) {
    let app = app(pool);
    let user = account(&app, "ホーム登録", false).await;
    let (server, world) = official(&app).await;
    let profile: Uuid = sqlx::query_scalar("SELECT id FROM profiles WHERE account_id=$1")
        .bind(user.id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO game_sessions(account_id,profile_id,native_uuid,session_id,server_id,lease_until) VALUES($1,$2,$3,$4,$5,now()+interval '45 seconds')")
        .bind(user.id).bind(profile).bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(server).execute(&app.db).await.unwrap();
    let mut jobs = Vec::new();
    for name in ["自宅", "農場", "採掘場"] {
        jobs.push(id(
            &run(&app, &user, Command::HomeSet { name: name.into() }).await,
            "job_id",
        ));
    }
    for name in ["4つ目", "自宅"] {
        assert!(
            commands::execute(
                &app,
                &user,
                Request {
                    request_id: Uuid::new_v4(),
                    command: Command::HomeSet { name: name.into() }
                }
            )
            .await
            .is_err()
        );
    }
    let (status, body) = acknowledge(
        &app,
        server,
        jobs[0],
        "succeeded",
        json!({"effect":"committed","location":{"world_id":world,"x":1,"y":64,"z":1}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) =
        acknowledge(&app, server, jobs[1], "failed", json!({"effect":"none"})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    run(
        &app,
        &user,
        Command::HomeSet {
            name: "代わりのホーム".into(),
        },
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn identity_selection_archives_economy_keeps_daily_cap_and_scopes_team_chat(pool: PgPool) {
    let app = app(pool);
    let (server, _) = official(&app).await;
    let retained = account(&app, "連携元", false).await;
    let other = account(&app, "連携先", false).await;
    let leader1 = account(&app, "チーム1", false).await;
    let leader2 = account(&app, "チーム2", false).await;
    let team1 = run(
        &app,
        &leader1,
        Command::TeamCreate {
            name: "一つ目".into(),
        },
    )
    .await;
    let team2 = run(
        &app,
        &leader2,
        Command::TeamCreate {
            name: "二つ目".into(),
        },
    )
    .await;
    for (leader, member, team) in [
        (&leader1, &retained, id(&team1, "team_id")),
        (&leader2, &other, id(&team2, "team_id")),
    ] {
        let invitation = run(
            &app,
            leader,
            Command::Invite {
                kind: "team".into(),
                resource: team,
                target: member.id,
            },
        )
        .await;
        run(
            &app,
            member,
            Command::InviteRespond {
                id: id(&invitation, "id"),
                accept: true,
            },
        )
        .await;
    }
    fund(&app, &retained, 700).await;
    fund(&app, &other, 1300).await;
    let selected: Uuid = sqlx::query_scalar("SELECT id FROM profiles WHERE account_id=$1")
        .bind(other.id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO npc_daily(profile_id,day,coins) SELECT id,current_date,1200 FROM profiles WHERE account_id IN ($1,$2)")
        .bind(retained.id).bind(other.id).execute(&app.db).await.unwrap();
    let begin = run(&app, &retained, Command::LinkBegin).await;
    run(
        &app,
        &other,
        Command::LinkPresent {
            code: begin["code"].as_str().unwrap().into(),
        },
    )
    .await;
    let migration = run(
        &app,
        &retained,
        Command::LinkConfirm {
            id: id(&begin, "id"),
            selected_profile: selected,
        },
    )
    .await;
    let (status, body) = acknowledge(
        &app,
        server,
        id(&migration, "job_id"),
        "succeeded",
        json!({"effect":"committed","native_data_verified":true,"native_uuid":Uuid::new_v4()}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT balance FROM wallets WHERE owner=$1")
            .bind(retained.id)
            .fetch_one(&app.db)
            .await
            .unwrap(),
        1300
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT sum(balance)::bigint FROM wallets")
            .fetch_one(&app.db)
            .await
            .unwrap(),
        2000
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT coins FROM npc_daily WHERE profile_id=$1 AND day=current_date"
        )
        .bind(selected)
        .fetch_one(&app.db)
        .await
        .unwrap(),
        2000
    );
    let rooms: Vec<Uuid> =
        sqlx::query_scalar("SELECT room_id FROM room_members WHERE account_id=$1")
            .bind(retained.id)
            .fetch_all(&app.db)
            .await
            .unwrap();
    let former_room: Uuid = sqlx::query_scalar("SELECT room_id FROM teams WHERE id=$1")
        .bind(id(&team2, "team_id"))
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert!(!rooms.contains(&former_room));
}

#[sqlx::test(migrations = "../../migrations")]
async fn public_queries_and_csrf_use_real_database(pool: PgPool) {
    let app = app(pool);
    let user = account(&app, "新規参加", false).await;
    let admin = account(&app, "運営", true).await;
    for view in [
        "home",
        "play",
        "social",
        "life",
        "market",
        "adventure",
        "servers",
        "settings",
    ] {
        let (status, result) = http(
            &app,
            &user,
            "GET",
            &format!("/api/v1/view/{view}"),
            json!({}),
            false,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{view}: {result}");
    }
    assert_eq!(
        http(&app, &admin, "GET", "/api/v1/view/admin", json!({}), false)
            .await
            .0,
        StatusCode::OK
    );
    let request =
        json!({"request_id":Uuid::new_v4(),"command":{"type":"room_create","name":"集会所"}});
    assert_eq!(
        http(
            &app,
            &user,
            "POST",
            "/api/v1/commands",
            request.clone(),
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (status, body) = http(&app, &user, "POST", "/api/v1/commands", request, true).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        http(&app, &user, "GET", "/api/v1/view/admin", json!({}), false)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn social_privacy_and_report_evidence(pool: PgPool) {
    let app = app(pool);
    let alice = account(&app, "Alice", false).await;
    let bob = account(&app, "Bob", false).await;
    let admin = account(&app, "Admin", true).await;
    run(&app, &alice, Command::FriendRequest { target: bob.id }).await;
    run(
        &app,
        &bob,
        Command::FriendRespond {
            target: alice.id,
            accept: true,
        },
    )
    .await;
    let room = id(
        &run(&app, &alice, Command::DirectRoom { target: bob.id }).await,
        "room_id",
    );
    let message = run(
        &app,
        &bob,
        Command::MessageSend {
            room,
            body: "通報対象".into(),
        },
    )
    .await["message_id"]
        .as_i64()
        .unwrap();
    run(
        &app,
        &bob,
        Command::MessageSend {
            room,
            body: "提出しない私信".into(),
        },
    )
    .await;
    assert_eq!(
        http(
            &app,
            &admin,
            "GET",
            &format!("/api/v1/rooms/{room}/messages"),
            json!({}),
            false
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let report = id(
        &run(
            &app,
            &alice,
            Command::Report {
                target: Some(bob.id),
                reason: "検証".into(),
                message_ids: vec![message],
            },
        )
        .await,
        "report_id",
    );
    let (status, body) = http(
        &app,
        &admin,
        "GET",
        &format!("/api/v1/reports/{report}"),
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["evidence"].as_array().unwrap().len(), 1);
    assert_eq!(body["evidence"][0]["body"], "通報対象");
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM audit WHERE action='report.read'")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(count, 1);
    run(
        &app,
        &alice,
        Command::Block {
            target: bob.id,
            blocked: true,
        },
    )
    .await;
    assert!(
        commands::execute(
            &app,
            &bob,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::MessageSend {
                    room,
                    body: "blocked".into()
                }
            }
        )
        .await
        .is_err()
    );
    let (status, body) = http(
        &app,
        &alice,
        "GET",
        &format!("/api/v1/rooms/{room}/messages?q=対象"),
        json!({}),
        false,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["messages"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_personal_owner_id_cannot_invite_someone_to_a_team(pool: PgPool) {
    let app = app(pool);
    let alice = account(&app, "Alice", false).await;
    let bob = account(&app, "Bob", false).await;
    let result = commands::execute(
        &app,
        &alice,
        Request {
            request_id: Uuid::new_v4(),
            command: Command::Invite {
                kind: "team".into(),
                resource: alice.id,
                target: bob.id,
            },
        },
    )
    .await;
    assert!(
        result.is_err(),
        "non-team resource must not produce an invitation"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn team_permissions_and_single_membership(pool: PgPool) {
    let app = app(pool);
    let leader = account(&app, "Leader", false).await;
    let member = account(&app, "Member", false).await;
    let team = id(
        &run(
            &app,
            &leader,
            Command::TeamCreate {
                name: "Builders".into(),
            },
        )
        .await,
        "team_id",
    );
    let invite = id(
        &run(
            &app,
            &leader,
            Command::Invite {
                kind: "team".into(),
                resource: team,
                target: member.id,
            },
        )
        .await,
        "id",
    );
    run(
        &app,
        &member,
        Command::InviteRespond {
            id: invite,
            accept: true,
        },
    )
    .await;
    assert!(
        commands::execute(
            &app,
            &member,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::TeamCreate {
                    name: "Second".into()
                }
            }
        )
        .await
        .is_err()
    );
    assert!(
        commands::execute(
            &app,
            &member,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::WalletTransfer {
                    owner: Some(team),
                    target: member.id,
                    amount: 1
                }
            }
        )
        .await
        .is_err()
    );
    assert!(
        commands::execute(
            &app,
            &leader,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::TeamLeave
            }
        )
        .await
        .is_err()
    );
    run(
        &app,
        &leader,
        Command::TeamTransfer {
            team,
            target: member.id,
        },
    )
    .await;
    run(&app, &leader, Command::TeamLeave).await;
    let quota: i32 = sqlx::query_scalar("SELECT chunks FROM land_allowances WHERE owner=$1")
        .bind(team)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(quota, 16);
}

#[sqlx::test(migrations = "../../migrations")]
async fn competing_buyers_and_retried_payment_cannot_duplicate_assets(pool: PgPool) {
    let app = app(pool);
    let seller = account(&app, "Seller", false).await;
    let alice = account(&app, "Alice", false).await;
    let bob = account(&app, "Bob", false).await;
    fund(&app, &alice, 1000).await;
    fund(&app, &bob, 1000).await;
    let asset = Uuid::new_v4();
    sqlx::query("INSERT INTO assets(id,owner,kind,title,state,manifest) VALUES($1,$2,'items','Escrowed diamonds','escrowed','{\"items\":[\"DIAMOND\"]}')").bind(asset).bind(seller.id).execute(&app.db).await.unwrap();
    let listing = id(
        &run(&app, &seller, Command::ListingCreate { asset, price: 100 }).await,
        "listing_id",
    );
    let a = Request {
        request_id: Uuid::new_v4(),
        command: Command::ListingBuy {
            id: listing,
            owner: None,
        },
    };
    let b = Request {
        request_id: Uuid::new_v4(),
        command: Command::ListingBuy {
            id: listing,
            owner: None,
        },
    };
    let (ra, rb) = tokio::join!(
        commands::execute(&app, &alice, a.clone()),
        commands::execute(&app, &bob, b.clone())
    );
    assert_ne!(ra.is_ok(), rb.is_ok());
    let (buyer, request, original) = if let Ok(result) = ra {
        (&alice, a, result)
    } else {
        (&bob, b, rb.unwrap())
    };
    assert_eq!(
        commands::execute(&app, buyer, request.clone())
            .await
            .unwrap(),
        original
    );
    assert!(
        commands::execute(
            &app,
            buyer,
            Request {
                request_id: request.request_id,
                command: Command::ListingCancel { id: listing }
            }
        )
        .await
        .is_err()
    );
    let row=sqlx::query("SELECT (SELECT count(*) FROM trades)::bigint AS trades,(SELECT sum(balance)::bigint FROM wallets) AS total,(SELECT balance FROM wallets WHERE owner=$1) AS seller,(SELECT owner FROM assets WHERE id=$2) AS owner").bind(seller.id).bind(asset).fetch_one(&app.db).await.unwrap();
    assert_eq!(row.get::<i64, _>("trades"), 1);
    assert_eq!(row.get::<i64, _>("total"), 2000);
    assert_eq!(row.get::<i64, _>("seller"), 95);
    assert_eq!(row.get::<Uuid, _>("owner"), buyer.id);
    let unbalanced:i64=sqlx::query_scalar("SELECT count(*) FROM (SELECT l.id FROM ledger l JOIN ledger_entries e ON e.transaction_id=l.id WHERE l.kind='market' GROUP BY l.id HAVING sum(e.amount)<>0) q").fetch_one(&app.db).await.unwrap();
    assert_eq!(unbalanced, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn failed_purchase_rolls_back_and_claims_enforce_quota_overlap(pool: PgPool) {
    let app = app(pool);
    official(&app).await;
    let a = account(&app, "A", false).await;
    let b = account(&app, "B", false).await;
    let claim = run(
        &app,
        &a,
        Command::ClaimCreate {
            owner: None,
            name: "家".into(),
            min_x: 0,
            min_z: 0,
            max_x: 1,
            max_z: 1,
        },
    )
    .await;
    assert_eq!(claim["state"], "pending");
    for (who, x, z) in [(&b, 1, 1), (&a, 100, 100)] {
        let result = commands::execute(
            &app,
            who,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::ClaimCreate {
                    owner: None,
                    name: "重複か上限超過".into(),
                    min_x: x,
                    min_z: z,
                    max_x: x,
                    max_z: z,
                },
            },
        )
        .await;
        assert!(result.is_err());
    }
    let asset = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO assets(id,owner,kind,title,state) VALUES($1,$2,'items','item','escrowed')",
    )
    .bind(asset)
    .bind(a.id)
    .execute(&app.db)
    .await
    .unwrap();
    let listing = id(
        &run(&app, &a, Command::ListingCreate { asset, price: 100 }).await,
        "listing_id",
    );
    assert!(
        commands::execute(
            &app,
            &b,
            Request {
                request_id: Uuid::new_v4(),
                command: Command::ListingBuy {
                    id: listing,
                    owner: None
                }
            }
        )
        .await
        .is_err()
    );
    let state: String = sqlx::query_scalar("SELECT state FROM listings WHERE id=$1")
        .bind(listing)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(state, "active");
}

#[sqlx::test(migrations = "../../migrations")]
async fn concurrent_spawns_are_distant_and_retry_reuses_reservation(pool: PgPool) {
    let app = app(pool);
    let (server, _world) = official(&app).await;
    let a = account(&app, "A", false).await;
    let b = account(&app, "B", false).await;
    let (a_spawn, b_spawn) = tokio::join!(
        internal(
            &app,
            "official",
            Some(server),
            "/internal/v1/spawn/reserve",
            json!({"account_id":a.id,"reason":"first_join"})
        ),
        internal(
            &app,
            "official",
            Some(server),
            "/internal/v1/spawn/reserve",
            json!({"account_id":b.id,"reason":"first_join"})
        )
    );
    assert_eq!(a_spawn.0, StatusCode::OK, "{}", a_spawn.1);
    assert_eq!(b_spawn.0, StatusCode::OK, "{}", b_spawn.1);
    let dx = a_spawn.1["x"].as_i64().unwrap() - b_spawn.1["x"].as_i64().unwrap();
    let dz = a_spawn.1["z"].as_i64().unwrap() - b_spawn.1["z"].as_i64().unwrap();
    assert!(dx * dx + dz * dz >= 100_000_000);
    let retry = internal(
        &app,
        "official",
        Some(server),
        "/internal/v1/spawn/reserve",
        json!({"account_id":a.id,"reason":"first_join"}),
    )
    .await;
    assert_eq!(retry.1["id"], a_spawn.1["id"]);
    assert_eq!(
        internal(
            &app,
            "proxy",
            None,
            "/internal/v1/spawn/reserve",
            json!({"account_id":a.id,"reason":"first_join"})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let bad = internal(
        &app,
        "official",
        Some(server),
        "/internal/v1/spawn/resolve",
        json!({"id":a_spawn.1["id"],"account_id":a.id,"state":"used"}),
    )
    .await;
    assert_eq!(bad.0, StatusCode::CONFLICT);
}
