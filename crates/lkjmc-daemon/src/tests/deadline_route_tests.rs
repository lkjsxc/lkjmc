use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::Service;
use uuid::Uuid;

use crate::app::{Admission, AppState};

#[derive(Clone, Copy)]
enum Route {
    Tcp,
    Web,
}
enum Fault {
    QueryCanceled,
    LockUnavailable,
}

#[test]
fn tcp_route_normalizes_real_database_deadlines() -> Result<(), String> {
    run_route_faults(Route::Tcp)
}

#[test]
fn web_route_normalizes_real_database_deadlines() -> Result<(), String> {
    run_route_faults(Route::Web)
}

fn run_route_faults(route: Route) -> Result<(), String> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        eprintln!("SKIP route deadline test: LKJMC_STORE_TEST_DATABASE_URL is unset");
        return Ok(());
    };
    run_fault(&url, route, Fault::QueryCanceled)?;
    run_fault(&url, route, Fault::LockUnavailable)
}

fn run_fault(url: &str, route: Route, fault: Fault) -> Result<(), String> {
    let mut database = crate::test_database::migrate(url)?;
    let token = insert_credential(database.client_mut())?;
    let application_name = format!("lkjmc-deadline-{}", Uuid::new_v4().simple());
    let worker_url = lkjmc_store::pool::with_application_name(database.url(), &application_name);
    let state = Admission::with_test_deadline(Duration::from_secs(1), || state(&worker_url));
    state.set_test_lock_timeout(Duration::from_millis(500))?;
    let mut pool_client = state.database_connection()?;
    let schema: String = pool_client
        .query_one("select current_schema()", &[])
        .map_err(|error| error.to_string())?
        .get(0);
    let name: String = pool_client
        .query_one("show application_name", &[])
        .map_err(|error| error.to_string())?
        .get(0);
    assert_eq!(schema, database.schema());
    assert_eq!(name, application_name);
    drop(pool_client);
    let mut transaction = database
        .client_mut()
        .transaction()
        .map_err(|error| error.to_string())?;
    lock_route(&mut transaction)?;
    let state_guard = state.clone();
    let (sent, received) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let mut router = crate::transport::routes::router(state, true);
        let result = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|error| error.to_string())
            .and_then(|runtime| {
                runtime.block_on(async {
                    let response = router
                        .call(request(route, &token))
                        .await
                        .map_err(|error| error.to_string())?;
                    let status = response.status();
                    let body = axum::body::to_bytes(response.into_body(), 1024)
                        .await
                        .map_err(|error| error.to_string())?;
                    Ok((status, body))
                })
            });
        let _ = sent.send(result);
    });
    let pid = match waiting_backend(&mut transaction, &application_name) {
        Ok(pid) => pid,
        Err(error) => {
            let result = received
                .recv_timeout(Duration::from_secs(1))
                .map_err(|_| error.clone())?;
            worker
                .join()
                .map_err(|_| "route worker panicked".to_string())?;
            let (status, body) = result?;
            return Err(format!(
                "{error}; route returned {status}: {}",
                String::from_utf8_lossy(&body)
            ));
        }
    };
    if matches!(fault, Fault::QueryCanceled) {
        let cancelled: bool = transaction
            .query_one("select pg_cancel_backend($1)", &[&pid])
            .map_err(|error| error.to_string())?
            .get(0);
        assert!(
            cancelled,
            "PostgreSQL did not cancel the blocked auth query"
        );
    }
    let result = received
        .recv_timeout(Duration::from_secs(3))
        .map_err(|_| "route did not return after PostgreSQL fault".to_string())?;
    worker
        .join()
        .map_err(|_| "route worker panicked".to_string())?;
    drop(state_guard);
    drop(transaction);
    let (status, body) = result?;
    assert_eq!(status, StatusCode::REQUEST_TIMEOUT);
    let value: Value = serde_json::from_slice(&body).map_err(|error| error.to_string())?;
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "command.deadline_exceeded");
    Ok(())
}

fn insert_credential(client: &mut postgres::Client) -> Result<String, String> {
    let token = format!("route-deadline-token-{}", Uuid::new_v4());
    lkjmc_store::daemon_token::insert(
        client,
        Uuid::new_v4(),
        &lkjmc_core::security::token_hash(&token),
        "web",
        "operator",
        "route-test",
        &["lkjmc.admin.admin".into()],
        60,
    )
    .map(|_| token)
    .map_err(|error| error.to_string())
}

fn state(url: &str) -> AppState {
    AppState::with_config_path(
        Some(url.into()),
        1,
        "/config".into(),
        "/log".into(),
        "/jars".into(),
        "/data".into(),
        None,
        None,
        None,
    )
}

fn request(route: Route, token: &str) -> Request<Body> {
    let builder =
        Request::builder().header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"));
    let builder = match route {
        Route::Tcp => builder.method("POST").uri("/command"),
        Route::Web => builder.method("GET").uri("/web/api/status"),
    };
    builder
        .body(Body::from("{}"))
        .unwrap_or_else(|_| Request::new(Body::empty()))
}

fn lock_route(transaction: &mut postgres::Transaction<'_>) -> Result<(), String> {
    transaction
        .query_one(
            "select revision from daemon_token_revision where singleton = true for update",
            &[],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn waiting_backend(
    transaction: &mut postgres::Transaction<'_>,
    application_name: &str,
) -> Result<i32, String> {
    let query = "select min(activity.pid)::int
                 from pg_stat_activity activity
                 join pg_locks lock on lock.pid = activity.pid
                 where activity.application_name = $1 and not lock.granted";
    for _ in 0..200 {
        let row = transaction
            .query_one(query, &[&application_name])
            .map_err(|error| error.to_string())?;
        if let Some(pid) = row.get::<_, Option<i32>>(0) {
            return Ok(pid);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("route database query did not reach its PostgreSQL lock".into())
}
