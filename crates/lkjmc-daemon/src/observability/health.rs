use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::app::AppState;
use crate::authz::AuthenticatedSubject;

pub(crate) async fn live() -> impl IntoResponse {
    Json(json!({"live": true, "source": "daemon-local"}))
}

pub(crate) async fn readiness(
    State(state): State<AppState>,
    subject: Option<Extension<AuthenticatedSubject>>,
    admission: Option<Extension<crate::app::RequestAdmission>>,
) -> Response {
    if !super::api::authorized(subject) {
        return super::api::forbidden();
    }
    let Some(Extension(admission)) = admission else {
        return unavailable("admission_unavailable");
    };
    match admission.run_blocking(move || readiness_body(&state)).await {
        Ok(Ok(body)) if body["ready"] == Value::Bool(true) => {
            (StatusCode::OK, Json(body)).into_response()
        }
        Ok(Ok(body)) => (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response(),
        Ok(Err(code)) => unavailable(code),
        Err(crate::app::BlockingError::Deadline) => unavailable("readiness_deadline"),
        Err(crate::app::BlockingError::Join) => unavailable("readiness_worker_failed"),
    }
}

pub(crate) fn readiness_body(state: &AppState) -> Result<Value, &'static str> {
    readiness_body_inner(state, None)
}

pub(crate) fn readiness_body_with_budget(
    state: &AppState,
    budget: std::time::Duration,
) -> Result<Value, &'static str> {
    readiness_body_inner(state, Some(budget))
}

fn readiness_body_inner(
    state: &AppState,
    budget: Option<std::time::Duration>,
) -> Result<Value, &'static str> {
    let (admission_open, _) = state.admission_diagnostics();
    let maintenance = state.maintenance_diagnostics();
    let runtime = state.runtime_capabilities();
    let mut client = match budget {
        Some(value) => state.request_database_connection_with_budget(value),
        None => state.request_database_connection(),
    }
    .map_err(|_| "database_unavailable")?;
    let applied = lkjmc_store::migrate::applied_versions(&mut client)
        .map_err(|_| "migration_check_failed")?;
    let migrations_current = applied == lkjmc_store::migrate::embedded_versions();
    let runtime_ready = runtime.readiness && runtime.process_identity && runtime.recovery;
    let retention_ready = maintenance.running && maintenance.last_error.is_none();
    let ready = migrations_current && admission_open && runtime_ready && retention_ready;
    Ok(json!({
        "ready": ready, "source": "daemon-local",
        "database": {"connected": true, "migrationsCurrent": migrations_current},
        "admission": {"open": admission_open},
        "maintenance": {"running": maintenance.running, "lastErrorClass": maintenance.last_error},
        "runtime": {"capable": runtime_ready},
        "syncRetention": {"ready": retention_ready}
    }))
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
