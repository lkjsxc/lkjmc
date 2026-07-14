use axum::extract::{Extension, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::authz::AuthenticatedSubject;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct EventParameters {
    request_id: Option<String>,
    operation_id: Option<Uuid>,
    correlation_id: Option<Uuid>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleRequest {
    output: String,
}

pub(crate) async fn metrics(
    State(state): State<AppState>,
    subject: Option<Extension<AuthenticatedSubject>>,
) -> Response {
    if !authorized(subject) {
        return forbidden();
    }
    let (_, in_flight) = state.admission_diagnostics();
    (
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        state.metrics().export(in_flight),
    )
        .into_response()
}

pub(crate) async fn events(
    State(state): State<AppState>,
    subject: Option<Extension<AuthenticatedSubject>>,
    admission: Option<Extension<crate::app::RequestAdmission>>,
    Query(parameters): Query<EventParameters>,
) -> Response {
    if !authorized(subject) {
        return forbidden();
    }
    if filter_count(&parameters) > 1 || !safe_request_id(parameters.request_id.as_deref()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"errorClass":"invalid_filter"})),
        )
            .into_response();
    }
    let Some(Extension(admission)) = admission else {
        return unavailable("admission_unavailable");
    };
    let result = admission
        .run_blocking(move || {
            state.request_database_connection().and_then(|mut client| {
                lkjmc_store::observability::query(
                    &mut *client,
                    lkjmc_store::observability::EventQuery {
                        request_id: parameters.request_id.as_deref(),
                        operation_id: parameters.operation_id,
                        correlation_id: parameters.correlation_id,
                        limit: parameters.limit.unwrap_or(100),
                    },
                )
            })
        })
        .await;
    match result {
        Ok(Ok(events)) => (
            StatusCode::OK,
            Json(json!({"events":events,"source":"daemon-local"})),
        )
            .into_response(),
        Ok(Err(error)) if error.is_deadline() => (
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({"errorClass":"database_deadline"})),
        )
            .into_response(),
        Err(crate::app::BlockingError::Deadline) => (
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({"errorClass":"database_deadline"})),
        )
            .into_response(),
        _ => unavailable("database_unavailable"),
    }
}

pub(crate) async fn support_bundle(
    State(state): State<AppState>,
    subject: Option<Extension<AuthenticatedSubject>>,
    admission: Option<Extension<crate::app::RequestAdmission>>,
    Json(request): Json<BundleRequest>,
) -> Response {
    if !authorized(subject) {
        return forbidden();
    }
    let Some(Extension(admission)) = admission else {
        return unavailable("admission_unavailable");
    };
    let result = admission
        .run_blocking(move || {
            let result =
                crate::support::bundle::create(&state, std::path::Path::new(&request.output));
            state.metrics().bundle(result.is_ok());
            result
        })
        .await;
    match result {
        Ok(Ok(manifest)) => {
            (StatusCode::CREATED, Json(json!({"manifest":manifest}))).into_response()
        }
        Ok(Err(error)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"errorClass":"bundle_failed","message":error})),
        )
            .into_response(),
        Err(crate::app::BlockingError::Deadline) => unavailable("bundle_deadline"),
        Err(crate::app::BlockingError::Join) => unavailable("bundle_worker_failed"),
    }
}

pub(super) fn authorized(subject: Option<Extension<AuthenticatedSubject>>) -> bool {
    subject.is_some_and(|Extension(value)| value.allows_observability())
}

fn safe_request_id(value: Option<&str>) -> bool {
    value.is_none_or(|item| {
        !item.is_empty()
            && item.len() <= 128
            && item.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
            })
    })
}

pub(super) fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"errorClass":"observability_denied"})),
    )
        .into_response()
}

fn unavailable(code: &'static str) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "ready": false, "source": "daemon-local", "errorClass": code
        })),
    )
        .into_response()
}

fn filter_count(value: &EventParameters) -> usize {
    usize::from(value.request_id.is_some())
        + usize::from(value.operation_id.is_some())
        + usize::from(value.correlation_id.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_correlation_filter_only() {
        let value = EventParameters {
            request_id: Some("a".into()),
            operation_id: Some(Uuid::nil()),
            ..Default::default()
        };
        assert_eq!(filter_count(&value), 2);
    }
}
