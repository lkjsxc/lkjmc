mod apply;

use lkjmc_core::bootstrap::{plan_bootstrap, BootstrapProfile, BootstrapRequest};
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use lkjmc_core::config::{BedrockEntry, BedrockMode, JavaEntry, PluginsConfig};
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
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
    match request_from_body(&request.body, true) {
        Ok(bootstrap_request) => {
            let facts = crate::bootstrap_facts::gather(state);
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
    match status_body(state) {
        Ok(body) => api::ok(request, body),
        Err(error) => api::error(request, "bootstrap.status_failed", error, false),
    }
}

fn doctor(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let facts = crate::bootstrap_facts::gather(state);
    let bootstrap_request = BootstrapRequest {
        profile: BootstrapProfile::Playable,
        accept_minecraft_eula: false,
        java_entry: JavaEntry::default(),
        bedrock_entry: BedrockEntry::default(),
        plugin_policy: PluginsConfig::default(),
        dry_run: true,
    };
    let plan = plan_bootstrap(&bootstrap_request, &facts);
    api::ok(
        request,
        json!({"facts": facts, "diagnostics": plan.diagnostics, "outcome": plan.outcome}),
    )
}

fn status_body(state: &AppState) -> Result<Value, String> {
    let Some(database_url) = state.database_url() else {
        return Ok(json!({"profile":"playable","result":"database-unavailable"}));
    };
    let mut client =
        lkjmc_store::pool::connect(&database_url).map_err(|error| error.to_string())?;
    crate::instance_helpers::refresh_runtime(state, &mut client)?;
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
    Ok(json!({"profile":"playable","instances":instances,"plugins":plugins}))
}

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

pub(super) fn request_from_body(body: &Value, dry_run: bool) -> Result<BootstrapRequest, String> {
    let profile = body
        .get("profile")
        .and_then(Value::as_str)
        .unwrap_or("playable");
    if profile != "playable" {
        return Err(format!("unsupported bootstrap profile: {profile}"));
    }
    let mut bedrock_entry = BedrockEntry::default();
    if let Some(mode) = body.get("bedrock").and_then(Value::as_str) {
        bedrock_entry.mode = match mode {
            "auto" => BedrockMode::Auto,
            "enabled" => BedrockMode::Enabled,
            "disabled" => BedrockMode::Disabled,
            other => return Err(format!("unsupported bedrock mode: {other}")),
        };
    }
    Ok(BootstrapRequest {
        profile: BootstrapProfile::Playable,
        accept_minecraft_eula: body
            .get("acceptMinecraftEula")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        java_entry: JavaEntry::default(),
        bedrock_entry,
        plugin_policy: PluginsConfig::default(),
        dry_run,
    })
}
