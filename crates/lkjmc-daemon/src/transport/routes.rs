use std::time::Duration;

use axum::http::StatusCode;
use axum::middleware;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use tower_http::timeout::TimeoutLayer;

use crate::app::AppState;

pub(super) const BODY_LIMIT: usize = 1024 * 1024;
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
        .route("/command", post(super::command::handle))
        .route("/sync/snapshot", post(super::sync::snapshot))
        .route("/sync/feed", post(super::sync::feed))
        .route("/health/live", get(crate::observability::health::live))
        .route(
            "/health/ready",
            get(crate::observability::health::readiness),
        )
        .route("/metrics", get(crate::observability::api::metrics))
        .route(
            "/observability/events",
            get(crate::observability::api::events),
        )
        .route(
            "/support/bundle",
            post(crate::observability::api::support_bundle),
        );
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            super::admission::require,
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
    fn unix_timeout_covers_the_network_apply_budget() {
        assert!(UNIX_COMMAND_TIMEOUT > crate::command_lifecycle::NETWORK_APPLY_DEADLINE);
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

    #[tokio::test]
    async fn shared_admission_covers_auth_and_web() -> Result<(), String> {
        let state = state(Some("token"));
        let _permits = (0..crate::command_lifecycle::ADMISSION_LIMIT)
            .map(|_| state.admit_request())
            .collect::<Option<Vec<_>>>()
            .ok_or("admission did not fill")?;
        let command = router(state.clone(), true)
            .oneshot(command_request(Some("Bearer token"), "{}"))
            .await
            .map_err(|error| error.to_string())?;
        assert_eq!(command.status(), StatusCode::OK);
        let sync = router(state.clone(), true)
            .oneshot(sync_request(Some("Bearer token")))
            .await
            .map_err(|error| error.to_string())?;
        assert_eq!(sync.status(), StatusCode::SERVICE_UNAVAILABLE);
        let web = router(state, true)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/web/login")
                    .body(Body::empty())
                    .map_err(|error| error.to_string())?,
            )
            .await
            .map_err(|error| error.to_string())?;
        assert_eq!(web.status(), StatusCode::SERVICE_UNAVAILABLE);
        Ok(())
    }

    fn sync_request(token: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().method("POST").uri("/sync/feed");
        if let Some(token) = token {
            builder = builder.header(axum::http::header::AUTHORIZATION, token);
        }
        builder
            .body(Body::from("{\"cursor\":0,\"limit\":1}"))
            .unwrap_or_else(|_| Request::new(Body::empty()))
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
