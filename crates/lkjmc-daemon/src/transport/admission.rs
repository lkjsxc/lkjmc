use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use lkjmc_core::command::CommandResponse;
use lkjmc_core::id::CommandId;
use serde_json::json;
use tokio::time::timeout_at;

use crate::app::AppState;

pub async fn require(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let command = matches!(request.uri().path(), "/" | "/command");
    let Some(admission) = state.admit_request() else {
        state.metrics().admission_rejected();
        return rejected(
            command,
            None,
            "command.queue_full",
            "request admission is full",
        );
    };
    request.extensions_mut().insert(admission.clone());
    if command_handler_owns_deadline(request.uri().path()) {
        // The command handler decodes under the ordinary admission deadline, then grants the
        // longer budget only to an authorized local bootstrap.apply. The route timeout remains
        // the hard transport cap; wrapping here would truncate that decoded command budget.
        return next.run(request).await;
    }
    match timeout_at(admission.deadline(), next.run(request)).await {
        Ok(response) => response,
        Err(_) => rejected(
            command,
            admission.request_id(),
            "command.deadline_exceeded",
            "request deadline elapsed; query a known requestId for its durable outcome",
        ),
    }
}

fn command_handler_owns_deadline(path: &str) -> bool {
    matches!(path, "/" | "/command")
}

fn rejected(command: bool, request_id: Option<CommandId>, code: &str, message: &str) -> Response {
    if command {
        return (
            StatusCode::OK,
            Json(CommandResponse {
                request_id: request_id
                    .unwrap_or_else(|| CommandId::internal("transport-admission")),
                ok: false,
                body: None,
                error: Some(lkjmc_core::command::CommandErrorBody {
                    code: code.to_string(),
                    message: message.to_string(),
                    retryable: true,
                }),
            }),
        )
            .into_response();
    }
    let status = if code == "command.queue_full" {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::REQUEST_TIMEOUT
    };
    (status, Json(json!({"ok": false, "error": {"code": code}}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::command_handler_owns_deadline;

    #[test]
    fn only_command_routes_defer_to_the_decoded_command_budget() {
        assert!(command_handler_owns_deadline("/"));
        assert!(command_handler_owns_deadline("/command"));
        assert!(!command_handler_owns_deadline("/health/ready"));
        assert!(!command_handler_owns_deadline("/sync/feed"));
    }
}
