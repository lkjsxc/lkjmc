mod apply;
mod connection;
mod request;
#[cfg(test)]
mod tests;

use lkjmc_core::bootstrap::plan_bootstrap;
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::commands::adventure_confirmation;
use crate::dispatch as api;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    if !adventure_confirmation::accepted(&request.body) {
        return adventure_confirmation::required(request);
    }
    match request.command.as_str() {
        "bootstrap.plan" => plan(state, request),
        "bootstrap.apply" => apply::apply(state, request),
        "bootstrap.status" => status(state, request),
        "bootstrap.doctor" => doctor(state, request),
        _ => api::error(
            request,
            "command.unknown",
            "unknown bootstrap command",
            false,
        ),
    }
}

fn plan(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request::from_body(state, &request.body, true) {
        Ok(bootstrap_request) => {
            let facts = crate::commands::bootstrap_facts::gather(state);
            let plan = plan_bootstrap(&bootstrap_request, &facts);
            match serde_json::to_value(plan) {
                Ok(body) => api::ok(request, body),
                Err(error) => api::error(request, "bootstrap.encode", error.to_string(), false),
            }
        }
        Err(error) => api::error(request, "bootstrap.request", error, false),
    }
}

fn status(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match status_body(state, &request.body) {
        Ok(body) => api::ok(request, body),
        Err(error) => api::error(request, "bootstrap.status_failed", error, false),
    }
}

fn doctor(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request::from_body(state, &request.body, true) {
        Ok(bootstrap_request) => {
            let facts = crate::commands::bootstrap_facts::gather(state);
            let plan = plan_bootstrap(&bootstrap_request, &facts);
            api::ok(
                request,
                json!({"facts": facts, "connection": connection::body(state).unwrap_or_else(|_| json!({})), "diagnostics": plan.diagnostics, "outcome": plan.outcome}),
            )
        }
        Err(error) => api::error(request, "bootstrap.request", error, false),
    }
}

fn status_body(state: &AppState, body: &Value) -> Result<Value, String> {
    let connection = connection::body(state)?;
    let next = connection["java"]["next"].clone();
    let plan = plan_status(state, body);
    let Some(_database_url) = database_url(state)? else {
        return Ok(json!({
            "profile":"playable",
            "result":"database-unavailable",
            "connection":connection,
            "next":next,
            "plan":plan
        }));
    };
    let mut client = state.database_connection()?;
    crate::support::instance_helpers::refresh_runtime(state, &mut client)?;
    let rows = lkjmc_store::instance::list(&mut client).map_err(|error| error.to_string())?;
    let mut instances = Vec::new();
    let mut plugins = Vec::new();
    for row in rows {
        instances.push(instance_json(&mut client, &row)?);
        for plugin in lkjmc_store::plugin::list_installations(&mut client, &row.id)
            .map_err(|error| error.to_string())?
        {
            plugins.push(json!({
                "id": plugin.plugin_id,
                "target": plugin.instance_id,
                "state": "installed",
                "targetPath": plugin.target_path,
                "sha256": plugin.installed_sha256
            }));
        }
    }
    Ok(json!({
        "profile":"playable",
        "instances":instances,
        "plugins":plugins,
        "connection":connection,
        "next":next,
        "plan":plan
    }))
}

pub(super) fn database_url(state: &AppState) -> Result<Option<String>, String> {
    match state.database_url() {
        Some(url) if url.trim().is_empty() => Err("Database URL is empty".to_string()),
        value => Ok(value),
    }
}

fn plan_status(state: &AppState, body: &Value) -> Value {
    let request = match request::from_body(state, body, true) {
        Ok(request) => request,
        Err(error) => return json!({"error": error}),
    };
    let facts = crate::commands::bootstrap_facts::gather(state);
    let plan = plan_bootstrap(&request, &facts);
    json!({
        "outcome": plan.outcome,
        "diagnostics": plan.diagnostics,
        "effects": plan.effects
    })
}

#[cfg(test)]
#[path = "bootstrap_api_tests.rs"]
mod runtime_tests;

fn instance_json(
    client: &mut postgres::Client,
    row: &lkjmc_store::instance::InstanceRecord,
) -> Result<Value, String> {
    let config =
        lkjmc_store::instance::config(client, &row.id).map_err(|error| error.to_string())?;
    let server_port = config.and_then(|value| {
        value
            .get("serverPort")
            .and_then(Value::as_i64)
            .map(Value::from)
    });
    Ok(json!({
        "id": row.id,
        "kind": row.kind,
        "desiredState": row.desired_state,
        "observedState": row.observed_state,
        "healthy": row.healthy,
        "pid": row.pid,
        "port": server_port
    }))
}
