use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::json;
use uuid::Uuid;

use super::cancellable;
use crate::app::AppState;

#[test]
fn adventure_cancel_is_denied_before_session_work() -> Result<(), String> {
    let request = CommandEnvelope {
        request_id: CommandId::parse("request id", "adventure.session.cancel")
            .map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "fenced-test".to_string(),
        },
        command: "adventure.session.cancel".to_string(),
        body: json!({"sessionId":Uuid::new_v4().to_string(),"reason":"operator cancel"}),
    };
    let state = AppState::with_config_path(
        None,
        2,
        "/tmp/config".to_string(),
        "/tmp/log".to_string(),
        "/tmp/jars".to_string(),
        "/tmp/data".to_string(),
        None,
        None,
        None,
    );
    let response = crate::dispatch::dispatch(&state, request);
    assert_eq!(
        response.error.map(|error| error.code),
        Some("command.effect_denied".to_string())
    );
    Ok(())
}

#[test]
fn cancellation_only_allows_pre_active_sessions() {
    assert!(cancellable("ready"));
    assert!(!cancellable("active"));
}
