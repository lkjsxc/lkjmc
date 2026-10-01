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
    Uuid::parse_str(v[key].as_str().unwrap()).unwrap()
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
