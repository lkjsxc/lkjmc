use std::time::Duration;

use axum::http::StatusCode;
use axum::middleware;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::json;
use tower_http::timeout::TimeoutLayer;

use crate::app::AppState;

const BODY_LIMIT: usize = 1024 * 1024;
const TCP_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const UNIX_COMMAND_TIMEOUT: Duration = Duration::from_secs(31 * 60);

pub fn router(state: AppState, tcp: bool) -> Router {
    let command_timeout = if tcp {
        TCP_COMMAND_TIMEOUT
    } else {
        UNIX_COMMAND_TIMEOUT
    };
    let mut command_routes = Router::new()
        .route("/", post(super::command::handle))
        .route("/command", post(super::command::handle));
    command_routes = if tcp {
        command_routes.route_layer(middleware::from_fn_with_state(
            state.clone(),
            super::auth::require_credential,
        ))
    } else {
        command_routes.route_layer(middleware::from_fn_with_state(
            state.clone(),
            super::peer::require_unix_peer,
        ))
    };
    command_routes
        .merge(crate::web::routes::router(state.clone()))
        .fallback(not_found)
        .layer(axum::extract::DefaultBodyLimit::max(BODY_LIMIT))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            command_timeout,
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
    use axum::http::Request;
    use tower::ServiceExt;

    #[test]
    fn unix_timeout_covers_the_readiness_window() {
        assert!(UNIX_COMMAND_TIMEOUT > Duration::from_secs(30 * 60));
        assert_eq!(TCP_COMMAND_TIMEOUT, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn tcp_command_denies_missing_or_bootstrap_secret() -> Result<(), String> {
        for header in [None, Some("Bearer bootstrap-secret")] {
            let response = router(state(Some("bootstrap-secret")), true)
                .oneshot(command_request(header, "{}"))
                .await
                .map_err(|error| error.to_string())?;
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
        Ok(())
    }

    #[tokio::test]
    async fn unix_command_requires_kernel_peer_extension() -> Result<(), String> {
        let response = router(state(None), false)
            .oneshot(command_request(None, "{}"))
            .await
            .map_err(|error| error.to_string())?;
        assert_ne!(response.status(), StatusCode::OK);
        Ok(())
    }

    fn command_request(token: Option<&str>, body: &str) -> Request<Body> {
        let mut builder = Request::builder().method("POST").uri("/command");
        if let Some(token) = token {
            builder = builder.header(axum::http::header::AUTHORIZATION, token);
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
