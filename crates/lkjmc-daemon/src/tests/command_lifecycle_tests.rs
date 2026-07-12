use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};

use crate::app::AppState;

const PLAYER: &str = "00000000-0000-0000-0000-000000000411";

#[test]
fn denied_effect_is_pre_handler() -> Result<(), String> {
    let response = crate::dispatch::dispatch(
        &state(None),
        request("instance.start", json!({"id":"hub"}))?,
    );
    assert!(!response.ok);
    assert_eq!(error_code(response), "command.effect_denied");
    Ok(())
}

#[test]
fn config_apply_truthful() -> Result<(), String> {
    let response = crate::dispatch::dispatch(&state(None), request("config.reload", json!({}))?);
    assert!(!response.ok);
    assert_eq!(error_code(response), "config.restart_required");
    Ok(())
}

#[test]
fn duplicate_mutations_pass() -> Result<(), String> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut database = crate::test_database::reset_and_migrate(&url)?;
    let state = state(Some(url));
    let body = json!({"playerUuid":PLAYER,"name":"Repeat","language":"ja"});
    let first = crate::dispatch::dispatch(&state, request("player.settings.set", body.clone())?);
    let second = crate::dispatch::dispatch(&state, request("player.settings.set", body)?);
    assert!(first.ok && second.ok);
    assert_eq!(first.body, second.body);
    let row = database
        .client_mut()
        .query_one(
            "select count(*)::bigint, max(language) from player_settings where player_uuid = $1",
            &[&uuid::Uuid::parse_str(PLAYER).map_err(|error| error.to_string())?],
        )
        .map_err(|error| error.to_string())?;
    assert_eq!(row.get::<_, i64>(0), 1);
    assert_eq!(row.get::<_, String>(1), "ja");
    Ok(())
}

#[test]
fn timeout_outcome_pass() -> Result<(), String> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let _database = crate::test_database::migrate(&url)?;
    let state = state(Some(url));
    let mut client = state.database_connection()?;
    let error = match client.batch_execute("select pg_sleep(6)") {
        Ok(()) => return Err("statement deadline did not stop PostgreSQL work".to_string()),
        Err(error) => error,
    };
    assert_eq!(
        error.code(),
        Some(&postgres::error::SqlState::QUERY_CANCELED),
        "database deadline returned an unexpected error: {error}"
    );
    Ok(())
}

fn error_code(response: lkjmc_core::command::CommandResponse) -> String {
    response.error.map(|error| error.code).unwrap_or_default()
}

fn request(command: &str, body: Value) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", command).map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "lifecycle-test".to_string(),
        },
        command: command.to_string(),
        body,
    })
}

fn state(database_url: Option<String>) -> AppState {
    AppState::with_config_path(
        database_url,
        8,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    )
}
