use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use postgres::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::runtime::RuntimeObservation;

pub fn with_client<F>(state: &AppState, request: CommandEnvelope, action: F) -> CommandResponse
where
    F: FnOnce(&AppState, CommandEnvelope, &mut Client) -> Result<CommandResponse, String>,
{
    let Some(database_url) = state.database_url() else {
        return api::error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    };
    let mut client = match lkjmc_store::pool::connect(&database_url) {
        Ok(client) => client,
        Err(error) => return api::error(request, "database.error", error.to_string(), false),
    };
    match action(state, request.clone(), &mut client) {
        Ok(response) => response,
        Err(error) => api::error(request, "instance.error", error, false),
    }
}

pub fn store<T>(result: Result<T, lkjmc_store::error::StoreError>) -> Result<T, String> {
    result.map_err(|error| error.to_string())
}

pub fn body_string(body: &Value, field: &'static str) -> Result<String, String> {
    body.get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("missing string field: {field}"))
}

pub fn create_config(body: &Value, template: &str) -> Value {
    let mut config = json!({"template": template});
    if let Some(command) = body.get("command").and_then(Value::as_str) {
        config["launch"] = json!({
            "command": "sh",
            "args": ["-c", command]
        });
    }
    if let Some(jar_asset_id) = body.get("jarAssetId").and_then(Value::as_str) {
        config["jarAssetId"] = Value::String(jar_asset_id.to_string());
    }
    if let Some(memory_mb) = body.get("memoryMb").and_then(Value::as_i64) {
        config["memoryMb"] = Value::Number(memory_mb.into());
    }
    if let Some(server_port) = body.get("serverPort").and_then(Value::as_i64) {
        config["serverPort"] = Value::Number(server_port.into());
    }
    if let Some(properties) = body.get("properties") {
        config["properties"] = properties.clone();
    }
    if let Some(files) = body.get("files") {
        config["files"] = files.clone();
    }
    if let Some(forwarding) = body.get("velocityForwardingMode") {
        config["velocityForwardingMode"] = forwarding.clone();
    }
    if let Some(rcon) = body.get("rcon") {
        config["rcon"] = rcon.clone();
    }
    config
}

pub fn launch(
    _state: &AppState,
    client: &mut Client,
    config: &Value,
) -> Result<(String, Vec<String>), String> {
    if let Some(asset_id) = config.get("jarAssetId").and_then(Value::as_str) {
        let asset_id = Uuid::parse_str(asset_id).map_err(|error| error.to_string())?;
        let memory_mb = config
            .get("memoryMb")
            .and_then(Value::as_i64)
            .unwrap_or(2048);
        return crate::jars::verified_launch(client, asset_id, memory_mb);
    }
    let launch = config
        .get("launch")
        .ok_or_else(|| "instance has no launch profile".to_string())?;
    let command = body_string(launch, "command")?;
    let args = launch
        .get("args")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    Ok((command, args))
}

pub fn runtime_start(
    state: &AppState,
    id: &str,
    command: &str,
    args: &[String],
    work_dir: &std::path::Path,
) -> Result<RuntimeObservation, String> {
    let mut runtime = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned".to_string())?;
    let log_root = state.log_root();
    runtime.start(id, command, args, &log_root, work_dir)
}

pub fn runtime_stop(state: &AppState, id: &str) -> Result<RuntimeObservation, String> {
    let mut runtime = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned".to_string())?;
    runtime.stop(id, std::time::Duration::from_secs(3))
}

pub fn runtime_running(state: &AppState, id: &str) -> Result<bool, String> {
    let mut runtime = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned".to_string())?;
    runtime.is_running(id)
}

pub fn start_runtime(
    state: &AppState,
    client: &mut Client,
    id: &str,
) -> Result<RuntimeObservation, String> {
    let instance = store(lkjmc_store::instance::get(client, id))?
        .ok_or_else(|| format!("instance not found: {id}"))?;
    let config = store(lkjmc_store::instance::config(client, id))?
        .ok_or_else(|| format!("instance not found: {id}"))?;
    let work_dir = crate::templates::render_instance(state, id, &instance.kind, &config)?;
    let (command, args) = launch(state, client, &config)?;
    let observation = runtime_start(state, id, &command, &args, &work_dir)?;
    write_observation(client, id, &observation)?;
    Ok(observation)
}

pub fn stop_runtime(
    state: &AppState,
    client: &mut Client,
    id: &str,
) -> Result<RuntimeObservation, String> {
    if let Some(config) = store(lkjmc_store::instance::config(client, id))? {
        let _ = crate::rcon::stop_from_config(&config);
    }
    let observation = runtime_stop(state, id)?;
    write_observation(client, id, &observation)?;
    Ok(observation)
}

pub fn write_observation(
    client: &mut Client,
    id: &str,
    observation: &RuntimeObservation,
) -> Result<(), String> {
    let pid = observation.pid.and_then(|pid| i32::try_from(pid).ok());
    lkjmc_store::instance::upsert_observation(
        client,
        id,
        &observation.observed_state,
        pid,
        observation.healthy,
        observation.message.as_deref(),
    )
    .map_err(|error| error.to_string())
}

pub fn refresh_runtime(state: &AppState, client: &mut Client) -> Result<(), String> {
    let instances = lkjmc_store::instance::list(client).map_err(|error| error.to_string())?;
    let mut runtime = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned".to_string())?;
    for instance in instances {
        if let Some(observation) = runtime.status(&instance.id)? {
            write_observation(client, &instance.id, &observation)?;
        }
    }
    Ok(())
}
