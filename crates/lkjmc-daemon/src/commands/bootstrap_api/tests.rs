use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};

use super::*;

#[test]
fn bootstrap_reads_work_without_a_request_scoped_legal_field() -> Result<(), String> {
    assert!(!handle(&state(), request("bootstrap.plan", json!({}))?).ok);
    for command in ["bootstrap.status", "bootstrap.doctor"] {
        assert!(handle(&state(), request(command, json!({}))?).ok);
    }
    Ok(())
}

fn request(command: &str, body: Value) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", command).map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "consent-test".to_string(),
        },
        command: command.to_string(),
        body,
    })
}

fn state() -> AppState {
    AppState::with_config_path(
        None,
        1,
        "/tmp/config".to_string(),
        "/tmp/logs".to_string(),
        "/tmp/jars".to_string(),
        "/tmp/data".to_string(),
        None,
        None,
        None,
    )
}
