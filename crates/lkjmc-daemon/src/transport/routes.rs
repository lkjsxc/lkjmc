use std::time::Duration;

use axum::http::StatusCode;
use axum::middleware;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::json;
use tower_http::timeout::TimeoutLayer;

use crate::app::AppState;

const BODY_LIMIT: usize = 1024 * 1024;

pub fn router(state: AppState, require_auth: bool) -> Router {
    let mut command_routes = Router::new()
        .route("/", post(super::command::handle))
        .route("/command", post(super::command::handle));
    if require_auth {
        command_routes = command_routes.route_layer(middleware::from_fn_with_state(
            state.clone(),
            super::auth::require_bearer,
        ));
    }
    command_routes
        .merge(crate::web_routes::router(state.clone()))
        .fallback(not_found)
        .layer(axum::extract::DefaultBodyLimit::max(BODY_LIMIT))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .with_state(state)
}

async fn not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"ok": false, "error": {"code": "route.not_found"}})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::header::AUTHORIZATION;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn tcp_command_requires_token() -> Result<(), String> {
        let response = router(state(Some("secret")), true)
            .oneshot(command_request(None, "{}"))
            .await
            .map_err(|error| error.to_string())?;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        Ok(())
    }

    #[tokio::test]
    async fn uds_command_allows_no_token() -> Result<(), String> {
        let response = router(state(None), false)
            .oneshot(command_request(None, "{}"))
            .await
            .map_err(|error| error.to_string())?;
        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn oversized_body_returns_413() -> Result<(), String> {
        let body = "x".repeat(BODY_LIMIT + 1);
        let response = router(state(Some("secret")), true)
            .oneshot(command_request(Some("Bearer secret"), &body))
            .await
            .map_err(|error| error.to_string())?;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        Ok(())
    }

    fn command_request(token: Option<&str>, body: &str) -> Request<Body> {
        let mut builder = Request::builder().method("POST").uri("/command");
        if let Some(token) = token {
            builder = builder.header(AUTHORIZATION, token);
        }
        builder
            .body(Body::from(body.to_string()))
            .unwrap_or_else(|_| Request::new(Body::empty()))
    }

    fn state(token: Option<&str>) -> AppState {
        AppState::with_config_path(
            None,
            8,
            "/tmp/lkjmc-config".to_string(),
            "/tmp/lkjmc-logs".to_string(),
            "/tmp/lkjmc-jars".to_string(),
            "/tmp/lkjmc-data".to_string(),
            None,
            None,
            token.map(ToString::to_string),
        )
    }
}
