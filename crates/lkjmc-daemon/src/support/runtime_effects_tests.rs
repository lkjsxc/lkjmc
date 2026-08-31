use serde_json::json;

use super::start_runtime;
use crate::app::AppState;
use crate::support::instance_helpers::stop_runtime;

#[test]
fn reconcile_retries_real_process_and_persists_identity() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut guard = crate::test_database::migrate(&database_url)?;
    let schema_url = guard.url().to_string();
    let root =
        std::env::temp_dir().join(format!("lkjmc-reconcile-{}", uuid::Uuid::new_v4().simple()));
    let instance_id = format!("reconcile-test-{}", uuid::Uuid::new_v4().simple());
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    crate::runtime::materialize_test_eula(&root.join("data"), &instance_id)?;
    let secret = root.join("forwarding.secret");
    std::fs::write(&secret, "test-forwarding-secret\n").map_err(|error| error.to_string())?;
    let make_state = || {
        AppState::with_config_path(
            Some(schema_url.clone()),
            2,
            root.join("config").to_string_lossy().into(),
            root.join("logs").to_string_lossy().into(),
            root.join("jars").to_string_lossy().into(),
            root.join("data").to_string_lossy().into(),
            None,
            None,
            None,
        )
    };
    let state = make_state();
    let config = json!({
        "template":"default", "serverPort":25577,
        "forwardingSecretFile":secret,
        "launch":{"command":"sleep","args":["300"]}
    });
    lkjmc_store::instance::insert(
        guard.client_mut(),
        &instance_id,
        None,
        "vanilla-custom",
        "running",
        &config,
    )
    .map_err(|error| error.to_string())?;

    let first = start_runtime(&state, &instance_id)?;
    let pid = first.pid().ok_or("started process identity missing")?;
    drop(state);
    let recovered_state = make_state();
    let second = start_runtime(&recovered_state, &instance_id)?;
    assert_eq!(second.pid(), Some(pid));
    let mut client = lkjmc_store::pool::connect(&schema_url).map_err(|error| error.to_string())?;
    let row = lkjmc_store::instance::get(&mut client, &instance_id)
        .map_err(|error| error.to_string())?
        .ok_or("reconcile instance missing")?;
    assert_eq!(row.pid, i32::try_from(pid).ok());
    drop(client);
    stop_runtime(&recovered_state, &instance_id)?;
    recovered_state.shutdown_runtime()?;
    drop(guard);
    let _ = std::fs::remove_dir_all(root);
    Ok(())
}
