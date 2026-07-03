use std::collections::HashMap;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::timeout::TimeoutLayer;

use crate::config::Config;

const BODY_LIMIT: usize = 1024 * 1024;

pub fn serve(addr: &str, config: Config) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("start discord runtime: {error}"))?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|error| format!("bind discord: {error}"))?;
        axum::serve(listener, router(config))
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|error| format!("serve discord: {error}"))
    })
}

fn router(config: Config) -> Router {
    Router::new()
        .route("/interactions", post(handle))
        .fallback(not_found)
        .layer(axum::extract::DefaultBodyLimit::max(BODY_LIMIT))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .with_state(config)
}

async fn handle(State(config): State<Config>, headers: HeaderMap, body: Bytes) -> Response {
    let headers = headers_map(&headers);
    let body = String::from_utf8_lossy(&body).to_string();
    let result = tokio::task::spawn_blocking(move || {
        crate::interaction::handle_interaction(&config, &headers, &body)
    })
    .await
    .unwrap_or_else(|error| (500, json!({"error": error.to_string()})));
    let status = StatusCode::from_u16(result.0).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(result.1)).into_response()
}

fn headers_map(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            Some((
                name.as_str().to_ascii_lowercase(),
                value.to_str().ok()?.to_string(),
            ))
        })
        .collect()
}

async fn not_found() -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
