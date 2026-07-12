use axum::body::Bytes;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;

use crate::app::{AppState, BlockingError, RequestAdmission};
use crate::authz::AuthenticatedSubject;
use crate::dispatch as api;

pub async fn handle(
    State(state): State<AppState>,
    subject: Option<Extension<AuthenticatedSubject>>,
    admission: Option<Extension<RequestAdmission>>,
    body: Bytes,
) -> Response {
    let Some(Extension(subject)) = subject else {
        return (
            StatusCode::FORBIDDEN,
            Json(api::error(
                invalid_request(),
                "auth.denied",
                "authentication required",
                false,
            )),
        )
            .into_response();
    };
    let Some(Extension(admission)) = admission else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(api::error(
                invalid_request(),
                "command.queue_full",
                "request admission is unavailable; no handler was invoked",
                true,
            )),
        )
            .into_response();
    };
    let response = match admission
        .run_blocking(move || match decode(&body) {
            Ok(envelope) => api::dispatch_as(&state, envelope, subject),
            Err(error) => api::error(invalid_request(), "request.invalid_json", error, false),
        })
        .await
    {
        Ok(response) => response,
        Err(BlockingError::Join) => api::error(
            invalid_request(),
            "request.dispatch_failed",
            "request worker failed",
            true,
        ),
        Err(BlockingError::Deadline) => api::error(
            invalid_request(),
            "command.deadline_exceeded",
            "command deadline elapsed; no completion result is available",
            true,
        ),
    };
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

#[cfg(test)]
mod tests {
    use crate::app::AppState;

    fn state() -> AppState {
        AppState::with_config_path(
            None,
            8,
            "/tmp/lkjmc-config".to_string(),
            "/tmp/lkjmc-logs".to_string(),
            "/tmp/lkjmc-jars".to_string(),
            "/tmp/lkjmc-data".to_string(),
            None,
            None,
            None,
        )
    }

    #[tokio::test]
    async fn queues_bounded() {
        let state = state();
        let permits = (0..crate::command_lifecycle::ADMISSION_LIMIT)
            .map(|_| state.admit_request())
            .collect::<Option<Vec<_>>>();
        assert!(permits.is_some());
        assert!(state.admit_request().is_none());
    }

    #[tokio::test]
    async fn command_load_budget_rejects_without_enqueuing() {
        let state = state();
        let mut permits = (0..crate::command_lifecycle::ADMISSION_LIMIT)
            .map(|_| state.admit_request())
            .collect::<Option<Vec<_>>>()
            .unwrap_or_default();
        assert!(state.admit_request().is_none());
        let permit = permits.pop();
        drop(permit);
        assert!(state.admit_request().is_some());
    }
}
