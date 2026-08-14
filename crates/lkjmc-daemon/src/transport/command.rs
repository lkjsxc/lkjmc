use std::time::Duration;

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use tokio::time::timeout_at;

use crate::app::{AppState, BlockingError, RequestAdmission};
use crate::authz::AuthenticatedSubject;
use crate::dispatch as api;

pub async fn handle(State(state): State<AppState>, request: Request<Body>) -> Response {
    let Some(subject) = request.extensions().get::<AuthenticatedSubject>().cloned() else {
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
    let Some(admission) = request.extensions().get::<RequestAdmission>().cloned() else {
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
    let body = match timeout_at(
        admission.deadline(),
        to_bytes(request.into_body(), super::routes::BODY_LIMIT),
    )
    .await
    {
        Ok(Ok(body)) => body,
        Ok(Err(error)) => {
            return (
                StatusCode::OK,
                Json(api::error(
                    invalid_request(),
                    "request.invalid_json",
                    format!("read request body: {error}"),
                    false,
                )),
            )
                .into_response()
        }
        Err(_) => {
            return (
                StatusCode::OK,
                Json(api::error(
                    invalid_request(),
                    "command.deadline_exceeded",
                    "command deadline elapsed before request body completed",
                    true,
                )),
            )
                .into_response()
        }
    };
    let decoded = admission.run_blocking(move || decode(&body)).await;
    let envelope = match decoded {
        Ok(Ok(envelope)) => envelope,
        Ok(Err(error)) => {
            return (
                StatusCode::OK,
                Json(api::error(
                    invalid_request(),
                    "request.invalid_json",
                    error,
                    false,
                )),
            )
                .into_response()
        }
        Err(BlockingError::Join) => {
            return (
                StatusCode::OK,
                Json(api::error(
                    invalid_request(),
                    "request.dispatch_failed",
                    "request decode worker failed",
                    true,
                )),
            )
                .into_response()
        }
        Err(BlockingError::Deadline) => {
            return (
                StatusCode::OK,
                Json(api::error(
                    invalid_request(),
                    "command.deadline_exceeded",
                    "command deadline elapsed before request decode completed",
                    true,
                )),
            )
                .into_response()
        }
    };
    admission.correlate(envelope.request_id.clone());
    let budget = command_budget(&subject, &envelope);
    let work = move || api::dispatch_as(&state, envelope, subject);
    let dispatched = match budget {
        Some(budget) => admission.run_blocking_with_budget(budget, work).await,
        None => admission.run_blocking(work).await,
    };
    let response = match dispatched {
        Ok(response) => response,
        Err(BlockingError::Join) => api::error(
            correlated_request(admission.request_id()),
            "request.dispatch_failed",
            "request worker failed",
            true,
        ),
        Err(BlockingError::Deadline) => api::error(
            correlated_request(admission.request_id()),
            "command.deadline_exceeded",
            "command deadline elapsed; query the durable outcome by requestId",
            true,
        ),
    };
    (StatusCode::OK, Json(response)).into_response()
}

fn command_budget(subject: &AuthenticatedSubject, envelope: &CommandEnvelope) -> Option<Duration> {
    (envelope.command == "bootstrap.apply" && subject.allows_local_runtime_effects())
        .then_some(crate::command_lifecycle::NETWORK_APPLY_DEADLINE)
}

fn decode(body: &[u8]) -> Result<CommandEnvelope, String> {
    serde_json::from_slice(body).map_err(|error| error.to_string())
}

fn invalid_request() -> CommandEnvelope {
    correlated_request(None)
}

fn correlated_request(request_id: Option<CommandId>) -> CommandEnvelope {
    CommandEnvelope {
        request_id: request_id.unwrap_or_else(|| CommandId::internal("http-decode-error")),
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
    use std::time::Duration;

    use axum::body::{to_bytes, Body};
    use axum::extract::State;
    use axum::http::Request;
    use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
    use lkjmc_core::id::CommandId;

    use super::{command_budget, handle};
    use crate::app::AppState;
    use crate::authz::AuthenticatedSubject;

    #[test]
    fn only_local_bootstrap_apply_receives_the_long_budget() {
        let request = CommandEnvelope {
            request_id: CommandId::internal("bootstrap-budget-test"),
            actor: Actor {
                kind: ActorKind::Cli,
                name: "untrusted".to_string(),
            },
            command: "bootstrap.apply".to_string(),
            body: serde_json::json!({}),
        };
        assert_eq!(
            command_budget(
                &AuthenticatedSubject::unix_peer(
                    crate::transport::peer::verified_unix_peer_for_test(1000),
                ),
                &request,
            ),
            Some(crate::command_lifecycle::NETWORK_APPLY_DEADLINE)
        );
        let remote_cli =
            AuthenticatedSubject::credential(lkjmc_store::daemon_token::DaemonTokenRecord {
                credential_id: uuid::Uuid::nil(),
                surface: "cli".to_string(),
                principal_kind: "operator".to_string(),
                principal_id: "remote-test".to_string(),
                scopes: vec!["lkjmc.admin.operator".to_string()],
                expires_at_micros: 1,
            });
        assert_eq!(command_budget(&remote_cli, &request), None);
        let mut status = request;
        status.command = "bootstrap.status".to_string();
        assert_eq!(
            command_budget(
                &AuthenticatedSubject::unix_peer(
                    crate::transport::peer::verified_unix_peer_for_test(1000),
                ),
                &status,
            ),
            None
        );
    }

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
    async fn request_body_collection_keeps_the_ordinary_deadline() -> Result<(), String> {
        let state = crate::app::Admission::with_test_deadline(Duration::from_millis(1), state);
        let admission = state
            .admit_request()
            .ok_or("request admission unavailable")?;
        tokio::time::sleep(Duration::from_millis(5)).await;
        let mut request = Request::builder()
            .body(Body::from("{}"))
            .map_err(|error| error.to_string())?;
        request
            .extensions_mut()
            .insert(AuthenticatedSubject::internal());
        request.extensions_mut().insert(admission);
        let response = handle(State(state), request).await;
        let body = to_bytes(response.into_body(), 4096)
            .await
            .map_err(|error| error.to_string())?;
        let response: CommandResponse =
            serde_json::from_slice(&body).map_err(|error| error.to_string())?;
        assert_eq!(
            response.error.as_ref().map(|error| error.code.as_str()),
            Some("command.deadline_exceeded")
        );
        Ok(())
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
