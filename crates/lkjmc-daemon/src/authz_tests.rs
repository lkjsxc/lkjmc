use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::StableId;
use uuid::Uuid;

use super::*;

#[test]
fn forged_actor_and_body_principals_do_not_change_credential_subject() -> Result<(), String> {
    let request = request(ActorKind::WebOperator);
    let contract = lkjmc_core::command_registry::contract_for("instance.delete")
        .ok_or_else(|| "missing instance.delete contract".to_string())?;
    let response = authorize(
        &state(),
        request,
        contract,
        &credential("web", vec!["lkjmc.admin.admin"]),
    );
    assert_eq!(
        response
            .err()
            .and_then(|value| value.error.map(|error| error.code)),
        Some("auth.surface_denied".into())
    );
    Ok(())
}

#[test]
fn credential_replaces_untrusted_actor_attribution() -> Result<(), String> {
    let contract = lkjmc_core::command_registry::contract_for("instance.delete")
        .ok_or_else(|| "missing instance.delete contract".to_string())?;
    let authorized = authorize(
        &state(),
        request(ActorKind::WebOperator),
        contract,
        &credential("cli", vec!["lkjmc.admin.admin"]),
    )
    .map_err(|_| "cli credential should authorize".to_string())?;
    assert_eq!(authorized.actor.kind, ActorKind::Cli);
    assert!(authorized.actor.name.starts_with("credential:"));
    assert_eq!(authorized.body["principalId"], "forged");
    Ok(())
}

#[test]
fn registry_policy_covers_every_registered_authorization_class() {
    for contract in lkjmc_core::command_registry::all() {
        assert!(matches!(
            contract.authorization.as_str(),
            "admin" | "operator" | "player"
        ));
        assert!(contract
            .surfaces
            .iter()
            .all(|surface| { matches!(surface.as_str(), "internal" | "cli" | "web") }));
    }
}

fn credential(surface: &str, scopes: Vec<&str>) -> AuthenticatedSubject {
    AuthenticatedSubject::credential(lkjmc_store::daemon_token::DaemonTokenRecord {
        credential_id: Uuid::nil(),
        surface: surface.into(),
        principal_kind: "operator".into(),
        principal_id: "stored-principal".into(),
        scopes: scopes.into_iter().map(str::to_string).collect(),
        expires_at_seconds: i64::MAX,
    })
}

fn request(kind: ActorKind) -> CommandEnvelope {
    CommandEnvelope {
        request_id: StableId::internal("test-command"),
        actor: Actor {
            kind,
            name: "forged".into(),
        },
        command: "instance.delete".into(),
        body: serde_json::json!({"id":"instance-1","principalId":"forged"}),
    }
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
