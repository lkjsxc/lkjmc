use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};

use super::*;

#[test]
fn bootstrap_eula_paths_use_the_shared_confirmation() -> Result<(), String> {
    for command in [
        "bootstrap.plan",
        "bootstrap.status",
        "bootstrap.doctor",
        "bootstrap.apply",
    ] {
        for body in [json!({}), json!({"acceptMinecraftEula": false})] {
            assert_confirmation(handle(&state(), request(command, body)?));
        }
    }
    Ok(())
}

fn assert_confirmation(response: CommandResponse) {
    assert!(!response.ok);
    assert!(response.body.is_none());
    assert_eq!(
        response.error.map(|error| (error.code, error.retryable)),
        Some((adventure_confirmation::CODE.to_string(), false))
    );
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
