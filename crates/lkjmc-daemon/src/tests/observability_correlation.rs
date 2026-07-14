use axum::body::{to_bytes, Body};
use axum::extract::Extension;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::app::AppState;
use crate::authz::AuthenticatedSubject;

#[test]
fn correlation_pass_uses_http_and_postgresql_thirty_times() -> Result<(), String> {
    let Some(database) = database()? else {
        return Ok(());
    };
    let state = state(Some(database.url().to_string()));
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    let router = runtime.block_on(correlation_http(state))?;
    drop(runtime);
    drop(router);
    drop(database);
    Ok(())
}

async fn correlation_http(state: AppState) -> Result<Router, String> {
    let admission = state.admit_request().ok_or("admission unavailable")?;
    let router = Router::new()
        .route("/command", post(crate::transport::command::handle))
        .route(
            "/observability/events",
            get(crate::observability::api::events),
        )
        .layer(Extension(AuthenticatedSubject::internal()))
        .layer(Extension(admission))
        .with_state(state);
    for repeat in 0..30 {
        let request_id = format!("obs-http-{repeat}");
        let envelope = CommandEnvelope {
            request_id: CommandId::parse("request id", request_id.clone())
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "untrusted-envelope".into(),
            },
            command: "status".into(),
            body: json!({}),
        };
        let command = Request::builder()
            .method("POST")
            .uri("/command")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&envelope).map_err(|error| error.to_string())?,
            ))
            .map_err(|error| error.to_string())?;
        let response = router
            .clone()
            .oneshot(command)
            .await
            .map_err(|error| error.to_string())?;
        assert_eq!(response.status(), StatusCode::OK);
        let query = Request::builder()
            .uri(format!("/observability/events?requestId={request_id}"))
            .body(Body::empty())
            .map_err(|error| error.to_string())?;
        let response = router
            .clone()
            .oneshot(query)
            .await
            .map_err(|error| error.to_string())?;
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 64 * 1024)
            .await
            .map_err(|error| error.to_string())?;
        let body: Value = serde_json::from_slice(&body).map_err(|error| error.to_string())?;
        let event = body["events"]
            .as_array()
            .and_then(|events| events.first())
            .ok_or("correlated event missing")?;
        assert_eq!(event["requestId"], request_id);
        assert_eq!(event["operationId"], event["eventId"]);
        assert_eq!(event["correlationId"], event["operationId"]);
        assert_eq!(event["source"], "daemon-local");
    }
    Ok(router)
}

#[test]
fn fault_diagnostics_pass_is_typed_http_non_success() -> Result<(), String> {
    let state = state(Some("not-a-postgres-url".into()));
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    let router = runtime.block_on(fault_http(state))?;
    drop(runtime);
    drop(router);
    Ok(())
}

async fn fault_http(state: AppState) -> Result<Router, String> {
    let admission = state.admit_request().ok_or("admission unavailable")?;
    let router = Router::new()
        .route(
            "/health/ready",
            get(crate::observability::health::readiness),
        )
        .layer(Extension(AuthenticatedSubject::internal()))
        .layer(Extension(admission))
        .with_state(state);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health/ready")
                .body(Body::empty())
                .map_err(|error| error.to_string())?,
        )
        .await
        .map_err(|error| error.to_string())?;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(response.into_body(), 4096)
        .await
        .map_err(|error| error.to_string())?;
    let text = String::from_utf8_lossy(&body);
    assert!(!text.contains("not-a-postgres-url"));
    let body: Value = serde_json::from_slice(&body).map_err(|error| error.to_string())?;
    assert_eq!(body["ready"], false);
    assert_eq!(body["errorClass"], "database_unavailable");
    Ok(router)
}

fn database() -> Result<Option<crate::test_database::TestDatabase>, String> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(None);
    };
    crate::test_database::migrate(&url).map(Some)
}

fn state(database_url: Option<String>) -> AppState {
    AppState::with_config_path(
        database_url,
        8,
        "/tmp/lkjmc-obs-config".into(),
        "/tmp/lkjmc-obs-logs".into(),
        "/tmp/lkjmc-obs-jars".into(),
        "/tmp/lkjmc-obs-data".into(),
        None,
        None,
        None,
    )
}
