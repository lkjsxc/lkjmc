use std::os::unix::process::CommandExt;

use serde_json::json;

use super::recover;
use crate::app::AppState;

#[test]
fn recovery_fences_live_pid_in_postgres() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut guard = reset_and_migrate(&database_url)?;
    let root = std::env::temp_dir().join(format!("lkjmc-recovery-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let state = AppState::with_config_path(
        Some(database_url),
        2,
        root.join("config").to_string_lossy().into(),
        root.join("logs").to_string_lossy().into(),
        root.join("jars").to_string_lossy().into(),
        root.join("data").to_string_lossy().into(),
        None,
        None,
        None,
    );
    let mut command = std::process::Command::new("sleep");
    command.arg("5").process_group(0);
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let result = (|| {
        lkjmc_store::instance::insert(
            &mut guard,
            "recovered",
            None,
            "paper",
            "running",
            &json!({}),
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::instance::upsert_observation(
            &mut guard,
            "recovered",
            "process-healthy",
            Some(i32::try_from(child.id()).map_err(|error| error.to_string())?),
            true,
            None,
        )
        .map_err(|error| error.to_string())?;
        recover(&state)?;
        let row = lkjmc_store::instance::get(&mut guard, "recovered")
            .map_err(|error| error.to_string())?
            .ok_or("recovered instance missing")?;
        assert_eq!(row.observed_state.as_deref(), Some("process-unhealthy"));
        assert_eq!(row.healthy, Some(false));
        assert!(row
            .message
            .is_some_and(|message| message.contains("fenced")));
        Ok(())
    })();
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(root);
    guard
        .batch_execute("select pg_advisory_unlock(752647)")
        .map_err(|error| error.to_string())?;
    result
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
