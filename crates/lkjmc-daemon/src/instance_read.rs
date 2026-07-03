use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, refresh_runtime, store, with_connection};

pub fn list(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_connection(state, request, |state, request, client| {
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
            let temporary = store(lkjmc_store::temporary::get_instance(client, &row.id))?;
            let join = joinability(
                row.healthy.unwrap_or(false),
                server_port.as_ref(),
                presence.as_ref(),
                proxy_registration(temporary.as_ref()),
            );
            instances.push(json!({
                "id": row.id,
                "kind": row.kind,
                "desiredState": row.desired_state,
                "observedState": row.observed_state,
                "healthy": row.healthy,
                "pid": row.pid,
                "serverPort": server_port,
                "connectHost": "127.0.0.1",
                "connectPort": server_port,
                "proxyRegistration": proxy_registration(temporary.as_ref()),
                "proxyRegistrationDesired": proxy_registration(temporary.as_ref()),
                "proxyRegistered": false,
                "joinable": join.0,
                "joinDisabledReason": join.1,
                "temporary": temporary.as_ref().map(|value| json!({
                    "lifecycleState": value.lifecycle_state,
                    "visibility": "hidden",
                    "cleanupPolicy": value.cleanup_policy,
                    "worldPath": value.world_path
                })),
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

fn joinability(
    healthy: bool,
    port: Option<&Value>,
    presence: Option<&lkjmc_store::instance_presence::PresenceRecord>,
    proxy_desired: bool,
) -> (bool, &'static str) {
    if !healthy {
        return (false, "server-unhealthy");
    }
    if port.is_none() {
        return (false, "missing-connect-port");
    }
    if !presence.map(|value| value.ready).unwrap_or(false) {
        return (false, "heartbeat-not-ready");
    }
    if proxy_desired {
        return (false, "proxy-registration-unknown");
    }
    (false, "proxy-registration-disabled")
}

fn proxy_registration(temporary: Option<&lkjmc_store::temporary::TemporaryInstanceRecord>) -> bool {
    temporary
        .map(|value| matches!(value.lifecycle_state.as_str(), "starting" | "ready"))
        .unwrap_or(true)
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
    let result = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned".to_string())
        .and_then(|mut runtime| runtime.logs(&id, &log_root, lines));
    match result {
        Ok(lines) => api::ok(request, json!({"id": id, "lines": lines})),
        Err(error) => api::error(request, "instance.logs_failed", error, false),
    }
}
