use serde_json::json;

use super::start_runtime;
use crate::app::AppState;
use crate::support::instance_helpers::runtime_stop;

#[test]
fn start_retries_a_real_process_effect_and_persists_health() -> Result<(), String> {
    let Ok(database_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut guard = crate::test_database::reset_and_migrate(&database_url)?;
    let root = std::env::temp_dir().join(format!("lkjmc-start-retry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let secret = root.join("forwarding.secret");
    std::fs::write(&secret, "test-forwarding-secret\n").map_err(|error| error.to_string())?;
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
    let result = (|| {
        let config = json!({
            "template":"default", "serverPort":25577, "eulaAccepted":true,
            "forwardingSecretFile":secret,
            "launch":{"command":"sh","args":["-c","if [ ! -f retry-marker ]; then touch retry-marker; exit 1; fi; while :; do sleep 1; done"]}
        });
        lkjmc_store::instance::insert(
            guard.client_mut(),
            "retry-test",
            None,
            "vanilla-custom",
            "stopped",
            &config,
        )
        .map_err(|error| error.to_string())?;
        let observation = start_runtime(&state, guard.client_mut(), "retry-test")?;
        assert!(observation.healthy);
        assert!(root.join("data/retry-test/retry-marker").exists());
        let row = lkjmc_store::instance::get(guard.client_mut(), "retry-test")
            .map_err(|error| error.to_string())?
            .ok_or("retry instance missing")?;
        assert_eq!(row.healthy, Some(true));
        runtime_stop(&state, "retry-test")?;
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(root);
    result
}
