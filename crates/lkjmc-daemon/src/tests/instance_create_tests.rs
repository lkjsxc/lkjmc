use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::json;

use crate::app::AppState;

#[test]
fn instance_create_is_denied_before_rcon_or_database_work() -> Result<(), String> {
    let root = std::env::temp_dir().join(format!(
        "lkjmc-rcon-create-{}",
        uuid::Uuid::new_v4().simple()
    ));
    let instance_id = format!("rcon-test-{}", uuid::Uuid::new_v4().simple());
    let _ = std::fs::remove_dir_all(&root);
    let state = AppState::with_config_path(
        None,
        2,
        root.to_string_lossy().into(),
        root.join("logs").to_string_lossy().into(),
        root.join("jars").to_string_lossy().into(),
        root.join("data").to_string_lossy().into(),
        None,
        None,
        None,
    );
    let response = crate::dispatch::dispatch(
        &state,
        CommandEnvelope {
            request_id: CommandId::parse("request id", "instance.create")
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "create-denial-test".to_string(),
            },
            command: "instance.create".to_string(),
            body: json!({
                "id":instance_id, "kind":"vanilla-custom", "template":"process-smoke",
                "command":"echo should-not-run",
                "rcon":{"port":25575,"password":"database-secret"}
            }),
        },
    );
    assert!(!response.ok);
    assert_eq!(
        response.error.map(|error| error.code),
        Some("command.effect_denied".to_string())
    );
    assert!(!root.exists());
    Ok(())
}
