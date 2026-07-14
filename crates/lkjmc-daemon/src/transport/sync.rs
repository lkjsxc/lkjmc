use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::app::{AppState, BlockingError, RequestAdmission};
use crate::authz::AuthenticatedSubject;

const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotRequest {
    domain: String,
    key: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeedRequest {
    cursor: i64,
    limit: i64,
}

pub async fn snapshot(
    State(state): State<AppState>,
    Extension(subject): Extension<AuthenticatedSubject>,
    Extension(admission): Extension<RequestAdmission>,
    Json(request): Json<SnapshotRequest>,
) -> Response {
    if !subject.allows_sync_read() {
        return error(StatusCode::FORBIDDEN, "auth.policy_denied");
    }
    run(admission, move || {
        let mut client = state.request_database_connection()?;
        let auth_revision = lkjmc_store::daemon_token::current_revision(&mut *client)?;
        Ok(
            match lkjmc_store::sync::snapshot(&mut client, &request.domain, &request.key)? {
                lkjmc_store::sync::SnapshotResult::Available(value) => json!({
                    "result": "snapshot", "domain": value.domain, "key": value.key,
                    "revision": value.revision, "generatedAt": value.generated_at,
                    "credentialRevision": auth_revision, "payload": value.payload
                }),
                lkjmc_store::sync::SnapshotResult::Unavailable { reason } => json!({
                    "result": "unavailable", "domain": request.domain, "key": request.key,
                    "credentialRevision": auth_revision, "reason": reason
                }),
            },
        )
    })
    .await
}

pub async fn feed(
    State(state): State<AppState>,
    Extension(subject): Extension<AuthenticatedSubject>,
    Extension(admission): Extension<RequestAdmission>,
    Json(request): Json<FeedRequest>,
) -> Response {
    if !subject.allows_sync_read() {
        return error(StatusCode::FORBIDDEN, "auth.policy_denied");
    }
    run(admission, move || {
        let mut client = state.request_database_connection()?;
        let auth_revision = lkjmc_store::daemon_token::current_revision(&mut *client)?;
        Ok(
            match lkjmc_store::sync::changes_after(&mut client, request.cursor, request.limit)? {
                lkjmc_store::sync::FeedResult::Changes {
                    changes,
                    cursor,
                    active_floor,
                } => json!({
                    "result": "changes", "cursor": cursor, "activeFloor": active_floor,
                    "credentialRevision": auth_revision,
                    "changes": changes.into_iter().map(|item| json!({
                        "feedRevision": item.feed_revision, "domain": item.domain,
                        "key": item.key, "revision": item.domain_revision
                    })).collect::<Vec<_>>()
                }),
                lkjmc_store::sync::FeedResult::ReloadRequired {
                    cursor,
                    active_floor,
                } => json!({
                    "result": "reload-required", "cursor": cursor,
                    "activeFloor": active_floor, "credentialRevision": auth_revision
                }),
            },
        )
    })
    .await
}

async fn run<F>(admission: RequestAdmission, work: F) -> Response
where
    F: FnOnce() -> Result<Value, lkjmc_store::error::StoreError> + Send + 'static,
{
    match admission.run_blocking(work).await {
        Ok(Ok(value)) => bounded(value),
        Ok(Err(store_error)) if store_error.is_deadline() => {
            error(StatusCode::REQUEST_TIMEOUT, "sync.deadline_exceeded")
        }
        Ok(Err(_)) => error(StatusCode::SERVICE_UNAVAILABLE, "sync.unavailable"),
        Err(BlockingError::Deadline) => {
            error(StatusCode::REQUEST_TIMEOUT, "sync.deadline_exceeded")
        }
        Err(BlockingError::Join) => error(StatusCode::SERVICE_UNAVAILABLE, "sync.unavailable"),
    }
}

fn bounded(value: Value) -> Response {
    if serde_json::to_vec(&value).map_or(true, |bytes| bytes.len() > MAX_RESPONSE_BYTES) {
        return error(StatusCode::SERVICE_UNAVAILABLE, "sync.response_too_large");
    }
    (StatusCode::OK, Json(value)).into_response()
}

fn error(status: StatusCode, code: &str) -> Response {
    (
        status,
        Json(json!({"result": "unavailable", "error": {"code": code}})),
    )
        .into_response()
}
