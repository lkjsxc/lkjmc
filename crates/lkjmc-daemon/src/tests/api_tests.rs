use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::json;

use crate::app::AppState;

#[test]
fn status_reports_running() -> Result<(), String> {
    let request = CommandEnvelope {
        request_id: CommandId::parse("request id", "test").map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "test".to_string(),
        },
        command: "status".to_string(),
        body: json!({}),
    };
    let response = crate::dispatch::dispatch(&state(), request);
    assert!(response.ok);
    let body = response
        .body
        .ok_or_else(|| "status body missing".to_string())?;
    assert_eq!(body["daemon"], json!("running"));
    assert_eq!(body["database"]["configured"], json!(false));
    Ok(())
}

fn state() -> AppState {
    AppState::with_config_path(
        None,
        8,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-test".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-instances".to_string(),
        None,
        None,
        None,
    )
}
