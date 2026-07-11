use super::wait_without_connection;
use crate::app::AppState;

#[test]
fn bootstrap_readiness_releases_pool_before_wait() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let state = AppState::with_config_path(
        Some(database_url),
        1,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    );
    let connection = state.database_connection()?;
    wait_without_connection(connection, || {
        let available = state.database_connection()?;
        drop(available);
        Ok(())
    })
}
