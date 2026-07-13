use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use postgres::{Client, Transaction};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::dispatch as api;
use crate::runtime::RuntimeObservation;

pub fn with_connection<F>(state: &AppState, request: CommandEnvelope, action: F) -> CommandResponse
where
    F: FnOnce(&AppState, CommandEnvelope, &mut Client) -> Result<CommandResponse, String>,
{
    let Some(pool) = state.database_pool() else {
        return api::error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    };
    let mut client = match pool.get() {
        Ok(client) => client,
        Err(error) => return api::error(request, "database.error", error.to_string(), false),
    };
    match action(state, request.clone(), &mut client) {
        Ok(response) => response,
        Err(error) => api::error(request, "instance.error", error, false),
    }
}

pub fn with_transaction<F>(state: &AppState, request: CommandEnvelope, action: F) -> CommandResponse
where
    F: FnOnce(&AppState, CommandEnvelope, &mut Transaction<'_>) -> Result<CommandResponse, String>,
{
    with_connection(state, request, |state, request, client| {
        let mut transaction = client.transaction().map_err(|error| error.to_string())?;
        let response = action(state, request, &mut transaction)?;
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(response)
    })
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
    if let Some(secret_file) = body.get("forwardingSecretFile") {
        config["forwardingSecretFile"] = secret_file.clone();
    }
    if let Some(env) = body.get("env") {
        config["env"] = env.clone();
    }
    config
}

pub fn runtime_running(state: &AppState, id: &str) -> Result<bool, String> {
    Ok(crate::runtime::reconcile::reconcile(
        state,
        id,
        crate::runtime::RuntimeGoal::Observe,
        uuid::Uuid::new_v4(),
    )?
    .healthy)
}

pub fn runtime_cancellation_state(state: &AppState, id: &str) -> Result<bool, String> {
    let observation = crate::runtime::reconcile::reconcile(
        state,
        id,
        crate::runtime::RuntimeGoal::Observe,
        uuid::Uuid::new_v4(),
    )?;
    if observation.healthy {
        Ok(true)
    } else if observation.observed_state == "process-absent" {
        Ok(false)
    } else {
        Err("runtime identity is unhealthy or fenced; refusing cancellation".to_string())
    }
}

pub(crate) use crate::support::runtime_effects::start_runtime;

pub fn stop_runtime(state: &AppState, id: &str) -> Result<RuntimeObservation, String> {
    crate::runtime::reconcile::reconcile(
        state,
        id,
        crate::runtime::RuntimeGoal::Stopped,
        uuid::Uuid::new_v4(),
    )
}

pub fn refresh_runtime(state: &AppState) -> Result<(), String> {
    let ids = {
        let mut client = state.database_connection()?;
        lkjmc_store::instance::list(&mut client)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|instance| instance.id)
            .collect::<Vec<_>>()
    };
    for id in ids {
        crate::runtime::reconcile::reconcile(
            state,
            &id,
            crate::runtime::RuntimeGoal::Observe,
            uuid::Uuid::new_v4(),
        )?;
    }
    Ok(())
}
