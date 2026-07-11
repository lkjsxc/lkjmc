use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};

use super::handle;
use crate::app::AppState;

#[test]
fn bootstrap_status_errors_for_empty_database_url_and_skips_when_unset() -> Result<(), String> {
    let empty = state(Some("   ".to_string()));
    let response = handle(
        &empty,
        request("bootstrap.status", json!({"acceptMinecraftEula": true})),
    );
    let error = response
        .error
        .ok_or("empty URL status unexpectedly succeeded")?;
    assert_eq!(error.code, "bootstrap.status_failed");
    assert_eq!(error.message, "Database URL is empty");

    let response = handle(
        &empty,
        request("bootstrap.apply", json!({"acceptMinecraftEula": true})),
    );
    let error = response
        .error
        .ok_or("empty URL apply unexpectedly succeeded")?;
    assert_eq!(error.code, "bootstrap.apply_failed");
    assert_eq!(error.message, "Database URL is empty");

    let unset = state(None);
    let response = handle(
        &unset,
        request("bootstrap.status", json!({"acceptMinecraftEula": true})),
    );
    assert!(response.ok);
    assert_eq!(
        response.body.as_ref().and_then(|body| body.get("result")),
        Some(&json!("database-unavailable"))
    );
    Ok::<(), String>(())
}

fn request(command: &str, body: Value) -> CommandEnvelope {
    CommandEnvelope {
        request_id: CommandId::internal("bootstrap-request"),
        actor: Actor {
            kind: ActorKind::Daemon,
            name: "test".to_string(),
        },
        command: command.to_string(),
        body,
    }
}

fn state(database_url: Option<String>) -> AppState {
    AppState::with_config_path(
        database_url,
        1,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    )
}
