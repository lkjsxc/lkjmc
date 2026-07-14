use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, refresh_runtime, store, with_connection};

const PROXY_REGISTRATION_TTL_SECONDS: i64 = 30;

pub fn list(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    if let Err(error) = refresh_runtime(state) {
        return api::error(request, "instance.error", error, false);
    }
    with_connection(state, request, |_state, request, client| {
        let rows = store(lkjmc_store::instance::list(client))?;
        let mut instances = Vec::new();
        for row in rows {
            let config = store(lkjmc_store::instance::config(client, &row.id))?;
            let server_port = config.as_ref().and_then(server_port);
            let configured_host = config
                .as_ref()
                .and_then(connect_host)
                .unwrap_or("127.0.0.1");
            let presence = store(lkjmc_store::instance_presence::get(client, &row.id))?;
            let temporary = store(lkjmc_store::temporary::get_instance(client, &row.id))?;
            let proxy_desired = proxy_registration(temporary.as_ref());
            let registration = store(lkjmc_store::proxy_registration::get(client, &row.id))?;
            let connect_host = registration
                .as_ref()
                .map(|value| value.connect_host.as_str())
                .unwrap_or(configured_host);
            let connect_port = registration
                .as_ref()
                .map(|value| i64::from(value.connect_port))
                .or(server_port);
            let join = joinability(
                row.healthy.unwrap_or(false),
                connect_port,
                presence.as_ref(),
                proxy_desired,
                registration.as_ref(),
            );
            instances.push(json!({
                "id": row.id,
                "kind": row.kind,
                "desiredState": row.desired_state,
                "observedState": row.observed_state,
                "healthy": row.healthy,
                "pid": row.pid,
                "serverPort": server_port,
                "connectHost": connect_host,
                "connectPort": connect_port,
                "proxyRegistration": proxy_desired,
                "proxyRegistrationDesired": proxy_desired,
                "proxyRegistered": registered(registration.as_ref()),
                "proxyRegistrationAgeSeconds": registration.as_ref().map(|value| value.age_seconds),
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
    port: Option<i64>,
    presence: Option<&lkjmc_store::instance_presence::PresenceRecord>,
    proxy_desired: bool,
    registration: Option<&lkjmc_store::proxy_registration::ProxyRegistrationRecord>,
) -> (bool, String) {
    if !healthy {
        return (false, "server-unhealthy".to_string());
    }
    if port.is_none() {
        return (false, "missing-connect-port".to_string());
    }
    if !presence.map(|value| value.ready).unwrap_or(false) {
        return (false, "heartbeat-not-ready".to_string());
    }
    if !proxy_desired {
        return (true, "".to_string());
    }
    let Some(registration) = registration else {
        return (false, "proxy-registration-unknown".to_string());
    };
    if registration.age_seconds > PROXY_REGISTRATION_TTL_SECONDS {
        return (false, "proxy-registration-stale".to_string());
    }
    if !registration.registered {
        return (
            false,
            registration
                .failure_reason
                .clone()
                .unwrap_or_else(|| "proxy-registration-failed".to_string()),
        );
    }
    (true, "".to_string())
}

fn registered(
    registration: Option<&lkjmc_store::proxy_registration::ProxyRegistrationRecord>,
) -> bool {
    registration
        .map(|value| value.registered && value.age_seconds <= PROXY_REGISTRATION_TTL_SECONDS)
        .unwrap_or(false)
}

fn connect_host(config: &Value) -> Option<&str> {
    config
        .get("connectHost")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
}

fn server_port(config: &Value) -> Option<i64> {
    config.get("serverPort").and_then(Value::as_i64)
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
    let runtime = state.runtime();
    let result = state.coordinate_runtime(&id, || runtime.runtime_logs(&id, &log_root, lines));
    match result {
        Ok(lines) => api::ok(request, json!({"id": id, "lines": lines})),
        Err(error) => api::error(request, "instance.logs_failed", error, false),
    }
}
