mod failure;
mod replay;
mod timeout;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use serde_json::json;

use crate::app::AppState;

const PLAYER: &str = "00000000-0000-0000-0000-000000000411";

fn request(id: &str, language: &str) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", id).map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "operation-test".into(),
        },
        command: "player.settings.set".into(),
        body: json!({"playerUuid": PLAYER, "name": "Repeat", "language": language}),
    })
}

async fn dispatch_admitted(
    state: &AppState,
    request: CommandEnvelope,
) -> Result<CommandResponse, String> {
    let admission = state
        .admit_request()
        .ok_or("request admission unavailable")?;
    let state = state.clone();
    admission
        .run_blocking(move || crate::dispatch::dispatch(&state, request))
        .await
        .map_err(|_| "request worker did not complete".to_string())
}

fn state(database_url: String) -> AppState {
    AppState::with_config_path(
        Some(database_url),
        8,
        "/tmp/lkjmc-config".into(),
        "/tmp/lkjmc-logs".into(),
        "/tmp/lkjmc-jars".into(),
        "/tmp/lkjmc-data".into(),
        None,
        None,
        None,
    )
}

fn database_url() -> Option<String> {
    std::env::var("LKJMC_STORE_TEST_DATABASE_URL").ok()
}

fn error_code(response: &CommandResponse) -> &str {
    response
        .error
        .as_ref()
        .map(|error| error.code.as_str())
        .unwrap_or_default()
}
