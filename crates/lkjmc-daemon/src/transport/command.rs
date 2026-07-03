use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;

use crate::api;
use crate::app::AppState;

pub async fn handle(State(state): State<AppState>, body: Bytes) -> Response {
    let response = tokio::task::spawn_blocking(move || match decode(&body) {
        Ok(envelope) => api::dispatch(&state, envelope),
        Err(error) => api::error(invalid_request(), "request.invalid_json", error, false),
    })
    .await
    .unwrap_or_else(|error| {
        api::error(
            invalid_request(),
            "request.dispatch_failed",
            error.to_string(),
            true,
        )
    });
    (StatusCode::OK, Json(response)).into_response()
}

fn decode(body: &[u8]) -> Result<CommandEnvelope, String> {
    serde_json::from_slice(body).map_err(|error| error.to_string())
}

fn invalid_request() -> CommandEnvelope {
    CommandEnvelope {
        request_id: CommandId::internal("http-decode-error"),
        actor: Actor {
            kind: ActorKind::Daemon,
            name: "lkjmc-daemon".to_string(),
        },
        command: "decode-error".to_string(),
        body: serde_json::json!({}),
    }
}
