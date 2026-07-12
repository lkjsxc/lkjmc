use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::json;

use crate::app::AppState;

#[test]
fn status_commands_share_bounded_pool() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let database = crate::test_database::migrate(&database_url)?;
    let state = AppState::with_config_path(
        Some(database.url().to_string()),
        2,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    runtime.block_on(async {
        assert!(call_status(&state).await?.ok);
        assert!(call_status(&state).await?.ok);
        let pool = state.database_pool().ok_or("missing database pool")?;
        let pool_state = pool.state();
        assert!(pool_state.connections >= 1);
        assert!(pool_state.connections <= state.database_pool_size());
        Ok(())
    })
}

async fn call_status(state: &AppState) -> Result<lkjmc_core::command::CommandResponse, String> {
    let admission = state
        .admit_request()
        .ok_or("request admission unavailable")?;
    let state = state.clone();
    let request = CommandEnvelope {
        request_id: CommandId::parse("request id", "status").map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "pool-test".to_string(),
        },
        command: "status".to_string(),
        body: json!({}),
    };
    admission
        .run_blocking(move || crate::dispatch::dispatch(&state, request))
        .await
        .map_err(|_| "request worker did not complete".to_string())
}
