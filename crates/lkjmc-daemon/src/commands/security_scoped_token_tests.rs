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
fn creation_and_revocation_audit_only_redacted_credential_data() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        eprintln!("SKIP scoped-token audit: LKJMC_STORE_TEST_DATABASE_URL is unset");
        return Ok(());
    };
    let mut database = crate::test_database::reset_and_migrate(&database_url)?;
    let root = std::env::temp_dir().join(format!("lkjmc-scoped-audit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let result =
        (|| {
            let state = database_state(database_url, &root);
            let mut create_request = request(json!(["lkjmc.admin.operator"]))?;
            let output = root.join("credential.token");
            create_request.body["outputFile"] = json!(output);
            create_request.body["principalId"] = json!("security-canary");
            let created = create(&state, create_request);
            assert!(created.ok);
            let body = created.body.as_ref().ok_or("credential response missing")?;
            for key in [
                "surface",
                "principalKind",
                "principalId",
                "scopes",
                "outputFile",
            ] {
                assert!(body.get(key).is_none(), "response leaked {key}");
            }
            let credential_id = body
                .get("credentialId")
                .and_then(serde_json::Value::as_str)
                .ok_or("credential id missing")?
                .to_string();
            let canary = std::fs::read_to_string(&output).map_err(|error| error.to_string())?;
            let mut revoke_request = request(json!([]))?;
            revoke_request.command = "security.daemon-token.revoke".into();
            revoke_request.body = json!({"credentialId": credential_id});
            assert!(revoke(&state, revoke_request).ok);
            let rows = database.client_mut().query(
            "select action, target_kind, target_id, result from audit_events order by action", &[],
        ).map_err(|error| error.to_string())?;
            assert_eq!(rows.len(), 2);
            let audit = rows
                .iter()
                .map(|row| {
                    format!(
                        "{}:{}:{}:{}",
                        row.get::<_, String>(0),
                        row.get::<_, String>(1),
                        row.get::<_, String>(2),
                        row.get::<_, String>(3)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            assert!(audit.contains("security.daemon-token.create:credential:"));
            assert!(audit.contains("security.daemon-token.revoke:credential:"));
            for forbidden in [
                canary.trim(),
                "security-canary",
                "lkjmc.admin.operator",
                output.to_str().ok_or("output path")?,
            ] {
                assert!(!audit.contains(forbidden));
            }
            Ok(())
        })();
    let _ = std::fs::remove_dir_all(root);
    result
}

#[test]
fn failed_create_preserves_an_existing_output_file() -> Result<(), String> {
    let root = std::env::temp_dir().join(format!("lkjmc-token-existing-{}", std::process::id()));
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let output = root.join("credential.token");
    std::fs::write(&output, "existing").map_err(|error| error.to_string())?;
    let error = match super::security_scoped_token_io::write_secret(
        output.to_str().ok_or("output path")?,
        "replacement",
    ) {
        Err(error) => error,
        Ok(()) => return Err("existing file accepted create".into()),
    };
    assert!(!error.created_file());
    assert_eq!(
        std::fs::read_to_string(&output).map_err(|error| error.to_string())?,
        "existing"
    );
    std::fs::remove_dir_all(root).map_err(|error| error.to_string())
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

fn database_state(database_url: String, root: &std::path::Path) -> AppState {
    AppState::with_config_path(
        Some(database_url),
        2,
        root.join("config").to_string_lossy().into(),
        root.join("log").to_string_lossy().into(),
        root.join("jars").to_string_lossy().into(),
        root.join("data").to_string_lossy().into(),
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
