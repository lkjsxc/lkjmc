use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, refresh_runtime, store, with_client};

pub fn list(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |state, request, client| {
        refresh_runtime(state, client)?;
        let rows = store(lkjmc_store::instance::list(client))?;
        let mut instances = Vec::new();
        for row in rows {
            let server_port =
                store(lkjmc_store::instance::config(client, &row.id))?.and_then(|config| {
                    config
                        .get("serverPort")
                        .and_then(Value::as_i64)
                        .map(Value::from)
                });
            let presence = store(lkjmc_store::instance_presence::get(client, &row.id))?;
            instances.push(json!({
                "id": row.id,
                "kind": row.kind,
                "desiredState": row.desired_state,
                "observedState": row.observed_state,
                "healthy": row.healthy,
                "pid": row.pid,
                "serverPort": server_port,
                "presence": presence.map(|value| json!({
                    "playerCount": value.player_count,
                    "maxPlayers": value.max_players,
                    "ready": value.ready,
                    "heartbeatAgeSeconds": value.heartbeat_age_seconds,
                    "emptySinceAgeSeconds": value.empty_since_age_seconds,
                    "suspendReason": value.suspend_reason
                }))
            }));
        }
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
    let log_root = state.log_root();
    match crate::logs::tail(&log_root, &id, lines) {
        Ok(lines) => api::ok(request, json!({"id": id, "lines": lines})),
        Err(error) => api::error(request, "instance.logs_failed", error, false),
    }
}
