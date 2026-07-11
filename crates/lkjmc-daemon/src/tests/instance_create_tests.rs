use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;

const SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn create_stores_only_an_rcon_password_reference() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut guard = crate::test_database::reset_and_migrate(&database_url)?;
    let root = std::env::temp_dir().join(format!("lkjmc-rcon-create-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let state = state_with_root(database_url, &root);
    let response = create(
        &state,
        json!({
            "id":"rcon-test", "kind":"vanilla-custom", "template":"process-smoke",
            "acceptMinecraftEula":true, "command":"while read line; do exit; done",
            "rcon":{"port":25575,"password":"database-secret"}
        }),
    )?;
    let result = (|| {
        assert!(response.ok);
        let config = lkjmc_store::instance::config(guard.client_mut(), "rcon-test")
            .map_err(|error| error.to_string())?
            .ok_or("RCON instance config missing")?;
        assert!(config["rcon"].get("password").is_none());
        assert!(!serde_json::to_string(&config)
            .map_err(|error| error.to_string())?
            .contains("database-secret"));
        let file = config["rcon"]["passwordFile"]
            .as_str()
            .ok_or("RCON password file missing")?;
        assert_eq!(
            std::fs::read_to_string(file).map_err(|error| error.to_string())?,
            "database-secret"
        );
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(root);
    result
}

#[test]
fn create_plan_reports_missing_jar_and_accepts_registered_asset() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut guard = crate::test_database::reset_and_migrate(&database_url)?;
    let state = state(database_url);
    let missing = call(&state, body())?;
    assert_eq!(missing["startable"], json!(false));
    assert_eq!(missing["diagnostic"]["code"], json!("jar_asset_missing"));
    assert_eq!(
        missing["diagnostic"]["suggestedCommand"],
        json!("lkjmc jar sync --project paper")
    );
    assert!(missing["diagnostic"]["attemptedQueries"]
        .as_array()
        .is_some_and(|values| !values.is_empty()));

    let asset_id = Uuid::new_v4();
    lkjmc_store::jar::insert(guard.client_mut(), jar(asset_id))
        .map_err(|error| error.to_string())?;
    let startable = call(&state, body())?;
    assert_eq!(startable["startable"], json!(true));
    assert_eq!(
        startable["createPlan"]["jarAssetId"],
        json!(asset_id.to_string())
    );
    Ok(())
}

fn body() -> Value {
    json!({
        "id": "hub",
        "kind": "paper",
        "template": "paper-survival",
        "acceptMinecraftEula": true
    })
}

fn call(state: &AppState, body: Value) -> Result<Value, String> {
    let response = create_command(state, "instance.create.plan", body)?;
    response.body.ok_or_else(|| "missing body".to_string())
}

fn create(state: &AppState, body: Value) -> Result<lkjmc_core::command::CommandResponse, String> {
    create_command(state, "instance.create", body)
}

fn create_command(
    state: &AppState,
    command: &str,
    body: Value,
) -> Result<lkjmc_core::command::CommandResponse, String> {
    Ok(crate::dispatch::dispatch(
        state,
        CommandEnvelope {
            request_id: CommandId::parse("request id", command)
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "create-plan-test".to_string(),
            },
            command: command.to_string(),
            body,
        },
    ))
}

fn state(database_url: String) -> AppState {
    state_with_root(database_url, std::path::Path::new("/tmp/lkjmc-config"))
}

fn state_with_root(database_url: String, root: &std::path::Path) -> AppState {
    AppState::with_config_path(
        Some(database_url),
        2,
        root.to_string_lossy().into(),
        root.join("logs").to_string_lossy().into(),
        root.join("jars").to_string_lossy().into(),
        root.join("data").to_string_lossy().into(),
        None,
        None,
        None,
    )
}

fn jar(id: Uuid) -> lkjmc_store::jar::NewJarAsset<'static> {
    lkjmc_store::jar::NewJarAsset {
        id,
        kind: "paper",
        project: "paper",
        channel: "stable",
        name: "paper-test.jar",
        path: "/opt/lkjmc/jars/paper-test.jar",
        sha256: SHA,
        size_bytes: 3,
        source: "test",
    }
}
