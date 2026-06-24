use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, refresh_runtime, store, with_client};

pub fn list(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |state, request, client| {
        refresh_runtime(state, client)?;
        let instances = store(lkjmc_store::instance::list(client))?
            .into_iter()
            .map(|row| {
                json!({
                    "id": row.id,
                    "kind": row.kind,
                    "desiredState": row.desired_state,
                    "observedState": row.observed_state,
                    "healthy": row.healthy,
                    "pid": row.pid
                })
            })
            .collect::<Vec<Value>>();
        Ok(api::ok(request, json!({"instances": instances})))
    })
}

pub fn logs(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    let id = match body_string(&request.body, "id") {
        Ok(value) => value,
        Err(error) => return api::error(request, "request.invalid", error, false),
    };
    let lines = request
        .body
        .get("lines")
        .and_then(Value::as_u64)
        .unwrap_or(120)
        .min(500) as usize;
    match crate::logs::tail(&state.log_root, &id, lines) {
        Ok(lines) => api::ok(request, json!({"id": id, "lines": lines})),
        Err(error) => api::error(request, "instance.logs_failed", error, false),
    }
}
