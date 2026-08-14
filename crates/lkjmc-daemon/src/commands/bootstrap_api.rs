mod apply;
mod connection;
mod network_state;
mod request;
#[cfg(test)]
mod tests;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::commands::adventure_confirmation;
use crate::dispatch as api;

const INSTANCE_CONFIG_SCHEMA_VERSION: u64 = 2;

fn heartbeat_endpoint(address: &str) -> String {
    let base = if address.starts_with("http://") || address.starts_with("https://") {
        address.to_string()
    } else {
        format!("http://{address}")
    };
    format!("{}/plugin/v1/heartbeat", base.trim_end_matches('/'))
}

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "bootstrap.plan" => plan(state, request),
        "bootstrap.apply" if !adventure_confirmation::accepted(&request.body) => {
            adventure_confirmation::required(request)
        }
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
    match network_state::inspect(state) {
        Ok(inspection) => match serde_json::to_value(inspection) {
            Ok(body) => api::ok(request, body),
            Err(error) => api::error(request, "bootstrap.encode", error.to_string(), false),
        },
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
    match request::validate(state, &request.body) {
        Ok(()) => {
            let inspection = network_state::inspect(state).ok();
            api::ok(
                request,
                json!({"connection": connection::body(state).unwrap_or_else(|_| json!({})), "inspection": inspection}),
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
    crate::support::instance_helpers::refresh_runtime(state)?;
    let mut client = state.database_connection()?;
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

fn plan_status(state: &AppState, _body: &Value) -> Value {
    network_state::inspect(state)
        .and_then(|inspection| serde_json::to_value(inspection).map_err(|error| error.to_string()))
        .unwrap_or_else(|error| json!({"error": error}))
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
