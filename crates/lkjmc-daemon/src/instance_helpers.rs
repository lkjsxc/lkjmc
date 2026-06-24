use lkjmc_core::command::{ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_store::audit::NewAuditEvent;
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
    let Some(database_url) = &state.database_url else {
        return api::error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    };
    let mut client = match lkjmc_store::pool::connect(database_url) {
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
    let launch = body.get("command").and_then(Value::as_str).map(|command| {
        json!({
            "command": "sh",
            "args": ["-c", command]
        })
    });
    match launch {
        Some(launch) => json!({"template": template, "launch": launch}),
        None => json!({"template": template}),
    }
}

pub fn launch(config: &Value) -> Result<(String, Vec<String>), String> {
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
) -> Result<RuntimeObservation, String> {
    let mut runtime = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned".to_string())?;
    runtime.start(id, command, args, &state.log_root)
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

pub fn audit(
    client: &mut Client,
    request: &CommandEnvelope,
    action: &str,
    target_kind: &str,
    target_id: &str,
    result: &str,
) -> Result<(), String> {
    lkjmc_store::audit::insert(
        client,
        NewAuditEvent {
            id: Uuid::new_v4(),
            actor_kind: actor_kind(request.actor.kind),
            actor_name: &request.actor.name,
            action,
            target_kind,
            target_id,
            result,
        },
    )
    .map_err(|error| error.to_string())
}

fn actor_kind(kind: ActorKind) -> &'static str {
    match kind {
        ActorKind::Cli => "cli",
        ActorKind::VelocityPlugin => "velocity-plugin",
        ActorKind::PaperPlugin => "paper-plugin",
        ActorKind::Daemon => "daemon",
        ActorKind::Installer => "installer",
    }
}
