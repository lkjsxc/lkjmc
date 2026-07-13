use std::thread;

use lkjmc_core::bootstrap::BootstrapEffect;
use lkjmc_core::id::InstanceId;
use serde_json::json;
use uuid::Uuid;

use super::{acquire_apply_lock, readiness_wait, steps};
use crate::app::AppState;

#[test]
fn readiness_records_before_pool_release_and_retains_lock_until_terminal_record(
) -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let database = crate::test_database::migrate(&database_url)?;
    let test_url = database.url().to_string();
    let state = AppState::with_config_path(
        Some(test_url.clone()),
        1,
        "/tmp/lkjmc-config".to_string(),
        "/tmp/lkjmc-logs".to_string(),
        "/tmp/lkjmc-jars".to_string(),
        "/tmp/lkjmc-data".to_string(),
        None,
        None,
        None,
    );
    let mut lock = acquire_apply_lock(&state)?;
    let run_id = Uuid::new_v4();
    let effect = BootstrapEffect::WaitForReadiness {
        id: InstanceId::internal("ledger-test"),
    };
    let connection = state.database_connection()?;
    let (step_id, waited) = readiness_wait::record_then_release(
        connection,
        |database| {
            lkjmc_store::bootstrap::create_run(
                database,
                lkjmc_store::bootstrap::NewBootstrapRun {
                    id: run_id,
                    profile: "playable",
                    requested_by: "test",
                    result: "running",
                    diagnostics: json!([]),
                },
            )
            .map_err(|error| error.to_string())?;
            steps::start(database, run_id, 0, &effect)
        },
        || {
            let mut available = state.database_connection()?;
            let steps = lkjmc_store::bootstrap::steps_for_run(&mut available, run_id)
                .map_err(|error| error.to_string())?;
            assert_eq!(steps.len(), 1);
            assert_eq!(steps[0].effect_kind, "probe.wait");
            assert_eq!(steps[0].result, "running");
            drop(available);
            let contender_url = test_url.clone();
            let contender = thread::spawn(move || -> Result<bool, String> {
                let mut client = lkjmc_store::pool::connect_single(&contender_url)
                    .map_err(|error| error.to_string())?;
                lkjmc_store::bootstrap::try_apply_lock(&mut client)
                    .map_err(|error| error.to_string())
            });
            let admitted = contender
                .join()
                .map_err(|_| "concurrent apply contender panicked".to_string())??;
            assert!(!admitted, "a concurrent apply acquired the dedicated lock");
            Ok(())
        },
    )
    .map_err(|error| {
        let (_, error) = *error;
        error
    })?;
    let mut reconnected = None;
    readiness_wait::reconnect_and_complete(
        &state,
        &mut reconnected,
        step_id,
        waited,
        steps::complete,
    )?;
    let database = reconnected
        .as_mut()
        .ok_or("terminal record did not reconnect to the pool")?;
    let steps = lkjmc_store::bootstrap::steps_for_run(database, run_id)
        .map_err(|error| error.to_string())?;
    assert_eq!(steps[0].result, "succeeded");
    drop(reconnected);
    lkjmc_store::bootstrap::release_apply_lock(&mut lock).map_err(|error| error.to_string())?;

    let mut contender =
        lkjmc_store::pool::connect_single(&test_url).map_err(|error| error.to_string())?;
    assert!(
        lkjmc_store::bootstrap::try_apply_lock(&mut contender).map_err(|error| error.to_string())?
    );
    lkjmc_store::bootstrap::release_apply_lock(&mut contender).map_err(|error| error.to_string())
}
