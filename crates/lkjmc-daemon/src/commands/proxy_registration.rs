use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn report(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let entries = request
            .body
            .get("registrations")
            .and_then(Value::as_array)
            .ok_or_else(|| "missing registrations array".to_string())?;
        let mut reports = Vec::new();
        for entry in entries {
            let object = entry
                .as_object()
                .ok_or_else(|| "registration must be object".to_string())?;
            let instance_id = object
                .get("instanceId")
                .and_then(Value::as_str)
                .unwrap_or("");
            let connect_host = object
                .get("connectHost")
                .and_then(Value::as_str)
                .unwrap_or("");
            let connect_port = object
                .get("connectPort")
                .and_then(Value::as_i64)
                .unwrap_or(0) as i32;
            if instance_id.is_empty() || connect_host.is_empty() || connect_port <= 0 {
                return Err("invalid proxy registration report".to_string());
            }
            reports.push(lkjmc_store::proxy_registration::RegistrationReport {
                instance_id,
                connect_host,
                connect_port,
                registered: object
                    .get("registered")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                failure_reason: object.get("failureReason").and_then(Value::as_str),
            });
        }
        let reported = reports.len();
        store(lkjmc_store::proxy_registration::report(client, &reports))?;
        drop(reports);
        let reporter =
            body_string(&request.body, "proxyId").unwrap_or_else(|_| "velocity".to_string());
        Ok(api::ok(
            request,
            json!({"proxyId": reporter, "reported": reported}),
        ))
    })
}
