use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::json;

use crate::app::AppState;

#[test]
fn status_commands_share_bounded_pool() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut guard = reset_and_migrate(&database_url)?;
    let state = AppState::with_config_path(
        Some(database_url),
        2,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    );
    assert!(call_status(&state)?.ok);
    assert!(call_status(&state)?.ok);
    let pool = state.database_pool().ok_or("missing database pool")?;
    let pool_state = pool.state();
    assert!(pool_state.connections >= 1);
    assert!(pool_state.connections <= state.database_pool_size());
    guard
        .batch_execute("select pg_advisory_unlock(752647)")
        .map_err(|error| error.to_string())?;
    Ok(())
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

fn call_status(state: &AppState) -> Result<lkjmc_core::command::CommandResponse, String> {
    Ok(crate::dispatch::dispatch(
        state,
        CommandEnvelope {
            request_id: CommandId::parse("request id", "status")
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "pool-test".to_string(),
            },
            command: "status".to_string(),
            body: json!({}),
        },
    ))
}
