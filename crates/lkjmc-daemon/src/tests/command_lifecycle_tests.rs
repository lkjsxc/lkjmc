use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};

use crate::app::AppState;

#[test]
fn denied_effect_is_pre_handler() -> Result<(), String> {
    let response = crate::dispatch::dispatch(
        &state(None),
        request("instance.start", json!({"id":"hub"}))?,
    );
    assert!(!response.ok);
    assert_eq!(error_code(response), "command.effect_denied");
    Ok(())
}

#[test]
fn config_apply_truthful() -> Result<(), String> {
    let response = crate::dispatch::dispatch(&state(None), request("config.reload", json!({}))?);
    assert!(!response.ok);
    assert_eq!(error_code(response), "config.restart_required");
    Ok(())
}

fn error_code(response: lkjmc_core::command::CommandResponse) -> String {
    response.error.map(|error| error.code).unwrap_or_default()
}

fn request(command: &str, body: Value) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", command).map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "lifecycle-test".to_string(),
        },
        command: command.to_string(),
        body,
    })
}

fn state(database_url: Option<String>) -> AppState {
    AppState::with_config_path(
        database_url,
        8,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    )
}
