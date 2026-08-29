use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::json;
use uuid::Uuid;

use super::local::LocalRuntime;
use super::process;
use crate::app::AppState;
use crate::runtime::test_support::{unique_id, StateCleanup};
use crate::support::instance_helpers::{start_runtime, stop_runtime};

pub(super) struct Fixture {
    pub(super) database: crate::test_database::TestDatabase,
    root: PathBuf,
}

impl Fixture {
    pub(super) fn new() -> Result<Option<Self>, String> {
        let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            return Ok(None);
        };
        let database = crate::test_database::migrate(&url)?;
        let root = std::env::temp_dir().join(format!(
            "lkjmc-runtime-adoption-{}-{}",
            std::process::id(),
            Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        Ok(Some(Self { database, root }))
    }

    pub(super) fn state(&self) -> AppState {
        AppState::with_config_path(
            Some(self.database.url().to_string()),
            8,
            self.path("config"),
            self.path("logs"),
            self.path("jars"),
            self.path("data"),
            None,
            None,
            None,
        )
    }

    pub(super) fn insert(&mut self, id: &str) -> Result<(), String> {
        let secret = self.root.join("forwarding.secret");
        std::fs::write(&secret, "test-forwarding-secret\n").map_err(|error| error.to_string())?;
        lkjmc_store::instance::insert(
            self.database.client_mut(),
            id,
            None,
            "vanilla-custom",
            "running",
            &json!({
                "template":"default", "serverPort":25577, "eulaAccepted":true,
                "forwardingSecretFile":secret,
                "launch":{"command":"sleep","args":["300"]}
            }),
        )
        .map_err(|error| error.to_string())
    }

    fn path(&self, child: &str) -> String {
        self.root.join(child).to_string_lossy().into()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn reconcile_idempotent_process_boundary() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new()? else {
        return Ok(());
    };
    let id = unique_id("idempotent");
    fixture.insert(&id)?;
    let state = std::sync::Arc::new(fixture.state());
    let _cleanup = StateCleanup(std::sync::Arc::clone(&state));
    let first = start_runtime(&state, &id)?;
    let pid = first.pid().ok_or("started process identity missing")?;
    let second = start_runtime(&state, &id)?;
    assert_eq!(second.pid(), Some(pid));
    let starts: i64 = fixture
        .database
        .client_mut()
        .query_one(
            "select count(*) from runtime_effect_workflows where instance_id=$1 and effect_kind='start'",
            &[&id],
        )
        .map_err(|error| error.to_string())?
        .get(0);
    assert_eq!(starts, 1);
    stop_runtime(&state, &id)?;
    state.shutdown_runtime()?;
    assert!(!process::group_exists(pid));
    Ok(())
}

#[test]
fn stale_persisted_database_identity_is_retired_before_restart() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new()? else {
        return Ok(());
    };
    let id = unique_id("stale-database-identity");
    fixture.insert(&id)?;
    let original = fixture.state();
    let first = start_runtime(&original, &id)?;
    let old_pid = first.pid().ok_or("started process identity missing")?;
    assert!(process::kill_group(old_pid));
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let observed = original
            .runtime()
            .runtime_status(&id)?
            .ok_or("post-kill process observation missing")?;
        if observed.observed_state.contains("absent") {
            break;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "killed process did not become absent before deadline: {}",
                observed.observed_state
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(original);

    let restarted = std::sync::Arc::new(fixture.state());
    let _cleanup = StateCleanup(std::sync::Arc::clone(&restarted));
    let repaired = start_runtime(&restarted, &id)?;
    let new_pid = repaired.pid().ok_or("restarted process identity missing")?;
    assert_ne!(new_pid, old_pid);
    stop_runtime(&restarted, &id)?;
    restarted.shutdown_runtime()?;
    assert!(!process::group_exists(new_pid));
    Ok(())
}

#[test]
fn effect_crash_recovery_process_boundary() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new()? else {
        return Ok(());
    };
    let intent_id = unique_id("after-intent");
    let effect_id = unique_id("after-effect");
    fixture.insert(&intent_id)?;
    fixture.insert(&effect_id)?;
    let intent = lkjmc_store::runtime_adoption::allocate(
        fixture.database.client_mut(),
        &intent_id,
        "start",
        &json!({"desired":"running"}),
        Uuid::new_v4(),
    )
    .map_err(|error| error.to_string())?;
    assert!(
        !lkjmc_store::runtime_adoption::pending(fixture.database.client_mut(), &intent_id)
            .map_err(|error| error.to_string())?
            .ok_or("intent crash row missing")?
            .effect_started
    );
    let state = std::sync::Arc::new(fixture.state());
    let _state_cleanup = StateCleanup(std::sync::Arc::clone(&state));
    let intent_observation = start_runtime(&state, &intent_id)?;
    let intent_pid = intent_observation
        .pid()
        .ok_or("intent recovery pid missing")?;
    assert!(
        lkjmc_store::runtime_adoption::history_count(fixture.database.client_mut(), intent.id)
            .map_err(|error| error.to_string())?
            >= 3
    );
    stop_runtime(&state, &intent_id)?;

    let operation = lkjmc_store::runtime_adoption::allocate(
        fixture.database.client_mut(),
        &effect_id,
        "start",
        &json!({"desired":"running"}),
        Uuid::new_v4(),
    )
    .map_err(|error| error.to_string())?;
    assert!(
        lkjmc_store::runtime_adoption::mark_effect(fixture.database.client_mut(), &operation)
            .map_err(|error| error.to_string())?
    );
    let data_root = fixture.root.join("data");
    let work_dir = data_root.join(&effect_id);
    std::fs::create_dir_all(&work_dir).map_err(|error| error.to_string())?;
    let crashed = LocalRuntime::with_data_root(&data_root);
    let effect = crashed.runtime_start(
        &effect_id,
        "sleep",
        &["300".to_string()],
        &BTreeMap::new(),
        &fixture.path("logs"),
        Path::new(&work_dir),
        Duration::from_secs(1),
    )?;
    let effect_pid = effect.pid().ok_or("effect crash pid missing")?;
    drop(crashed);
    let restarted = std::sync::Arc::new(fixture.state());
    let _restarted_cleanup = StateCleanup(std::sync::Arc::clone(&restarted));
    let repaired = start_runtime(&restarted, &effect_id)?;
    assert_eq!(repaired.pid(), Some(effect_pid));
    let stale = operation;
    let newer = lkjmc_store::runtime_adoption::allocate(
        fixture.database.client_mut(),
        &effect_id,
        "observe",
        &json!({"recover":true}),
        Uuid::new_v4(),
    )
    .map_err(|error| error.to_string())?;
    assert!(!lkjmc_store::runtime_adoption::observe(
        fixture.database.client_mut(),
        &stale,
        &json!({"observedState":"process-healthy","healthy":true}),
        "succeeded",
        Some("stale outcome")
    )
    .map_err(|error| error.to_string())?);
    assert!(newer.fence > stale.fence);
    start_runtime(&restarted, &effect_id)?;
    stop_runtime(&restarted, &effect_id)?;
    restarted.shutdown_runtime()?;
    assert!(!process::group_exists(intent_pid));
    assert!(!process::group_exists(effect_pid));
    Ok(())
}
