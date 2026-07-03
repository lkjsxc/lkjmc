use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;

const SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn create_plan_reports_missing_jar_and_accepts_registered_asset() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut guard = reset_and_migrate(&database_url)?;
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
    lkjmc_store::jar::insert(&mut guard, jar(asset_id)).map_err(|error| error.to_string())?;
    let startable = call(&state, body())?;
    assert_eq!(startable["startable"], json!(true));
    assert_eq!(
        startable["createPlan"]["jarAssetId"],
        json!(asset_id.to_string())
    );
    guard
        .batch_execute("select pg_advisory_unlock(752647)")
        .map_err(|error| error.to_string())?;
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
    let response = crate::dispatch::dispatch(
        state,
        CommandEnvelope {
            request_id: CommandId::parse("request id", "instance.create.plan")
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "create-plan-test".to_string(),
            },
            command: "instance.create.plan".to_string(),
            body,
        },
    );
    response.body.ok_or_else(|| "missing body".to_string())
}

fn state(database_url: String) -> AppState {
    AppState::with_config_path(
        Some(database_url),
        2,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    )
}

fn reset_and_migrate(database_url: &str) -> Result<postgres::Client, String> {
    let mut client =
        lkjmc_store::pool::connect_single(database_url).map_err(|error| error.to_string())?;
    client
        .batch_execute(
            "select pg_advisory_lock(752647); drop schema public cascade; create schema public",
        )
        .map_err(|error| error.to_string())?;
    lkjmc_store::migrate::apply(&mut client).map_err(|error| error.to_string())?;
    Ok(client)
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
