use std::fs;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::json;

use super::*;

#[test]
fn generated_token_is_secret_shaped() {
    let token = generate_token();
    assert!(token.len() >= 40);
    assert!(!token.contains('\n'));
}

#[test]
fn rotate_replaces_file_and_hot_swaps_token() -> Result<(), String> {
    let path = std::env::temp_dir().join(format!("lkjmc-rotate-{}.token", std::process::id()));
    fs::write(&path, "old-token\n").map_err(|error| error.to_string())?;
    let state = AppState::with_config_path(
        None,
        8,
        "/c".into(),
        "/l".into(),
        "/j".into(),
        "/d".into(),
        None,
        Some(path.to_string_lossy().to_string()),
        Some("old-token".into()),
    );
    let response = rotate(&state, request("security.daemon-token.rotate")?);
    assert!(response.ok);
    let new_token = state.http_token().ok_or("missing new token")?;
    assert_ne!(new_token, "old-token");
    assert!(crate::http_auth::authorized(
        &http(&new_token),
        Some(&new_token)
    ));
    assert!(!crate::http_auth::authorized(
        &http("old-token"),
        Some(&new_token)
    ));
    assert_eq!(
        fs::read_to_string(&path).map_err(|e| e.to_string())?.trim(),
        new_token
    );
    fs::remove_file(path).ok();
    Ok(())
}

fn request(command: &str) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", command).map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "test".into(),
        },
        command: command.into(),
        body: json!({}),
    })
}

fn http(token: &str) -> String {
    format!("POST / HTTP/1.1\r\nAuthorization: Bearer {token}\r\ncontent-length: 0\r\n\r\n")
}
