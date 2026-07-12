use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::json;

use super::*;

#[test]
fn scope_allowlist_rejects_unknown_and_mixed_values() {
    assert!(allowed_scopes(&["lkjmc.admin.operator".into()]));
    assert!(!allowed_scopes(&["unknown.scope".into()]));
    assert!(!allowed_scopes(&[
        "lkjmc.admin.operator".into(),
        "unknown.scope".into(),
    ]));
}

#[test]
fn creation_rejects_each_request_with_invalid_scope() -> Result<(), String> {
    for scopes in [
        json!(["unknown.scope"]),
        json!(["lkjmc.admin.status", "unknown.scope"]),
    ] {
        let response = create(&state(), request(scopes)?);
        assert!(!response.ok);
        assert_eq!(
            response.error.as_ref().map(|error| error.code.as_str()),
            Some("security.credential_invalid")
        );
    }
    Ok(())
}

#[test]
fn withdrawn_adapter_surface_is_unavailable() -> Result<(), String> {
    let mut request = request(json!(["lkjmc.admin.operator"]))?;
    request.body["surface"] = json!("paper");
    let response = create(&state(), request);
    assert_eq!(
        response.error.as_ref().map(|error| error.code.as_str()),
        Some("security.credential_invalid")
    );
    Ok(())
}

#[test]
fn known_scope_reaches_storage_validation() -> Result<(), String> {
    let response = create(&state(), request(json!(["lkjmc.admin.operator"]))?);
    assert!(!response.ok);
    assert_eq!(
        response.error.as_ref().map(|error| error.code.as_str()),
        Some("database.not_configured")
    );
    Ok(())
}

fn state() -> AppState {
    AppState::with_config_path(
        None,
        8,
        "/config".into(),
        "/log".into(),
        "/jars".into(),
        "/data".into(),
        None,
        None,
        None,
    )
}

fn request(scopes: serde_json::Value) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", "scoped-token")
            .map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "test".into(),
        },
        command: "security.daemon-token.create".into(),
        body: json!({
            "surface": "cli",
            "principalKind": "operator",
            "principalId": "player-1",
            "outputFile": "/tmp/scoped.token",
            "expiresInSeconds": 60,
            "scopes": scopes,
        }),
    })
}
