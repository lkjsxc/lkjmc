use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::json;
use uuid::Uuid;

use super::local::LocalRuntime;
use super::process;
use crate::app::AppState;
use crate::support::instance_helpers::{start_runtime, stop_runtime};

pub(super) struct Fixture {
    database: crate::test_database::TestDatabase,
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

    pub(super) fn client(&mut self) -> &mut postgres::Client {
        self.database.client_mut()
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
    fixture.insert("idempotent")?;
    let state = fixture.state();
    let first = start_runtime(&state, "idempotent")?;
    let pid = first.pid().ok_or("started process identity missing")?;
    let second = start_runtime(&state, "idempotent")?;
    assert_eq!(second.pid(), Some(pid));
    let starts: i64 = fixture
        .database
        .client_mut()
        .query_one(
            "select count(*) from runtime_effect_workflows where instance_id=$1 and effect_kind='start'",
            &[&"idempotent"],
        )
        .map_err(|error| error.to_string())?
        .get(0);
    assert_eq!(starts, 1);
    stop_runtime(&state, "idempotent")?;
    state.shutdown_runtime()?;
    assert!(!process::group_exists(pid));
    Ok(())
}

#[test]
fn effect_crash_recovery_process_boundary() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new()? else {
        return Ok(());
    };
    fixture.insert("after-intent")?;
    fixture.insert("after-effect")?;
    let intent = lkjmc_store::runtime_adoption::allocate(
        fixture.database.client_mut(),
        "after-intent",
        "start",
        &json!({"desired":"running"}),
        Uuid::new_v4(),
    )
    .map_err(|error| error.to_string())?;
    assert!(
        !lkjmc_store::runtime_adoption::pending(fixture.database.client_mut(), "after-intent")
            .map_err(|error| error.to_string())?
            .ok_or("intent crash row missing")?
            .effect_started
    );
    let state = fixture.state();
    let intent_observation = start_runtime(&state, "after-intent")?;
    let intent_pid = intent_observation
        .pid()
        .ok_or("intent recovery pid missing")?;
    assert!(
        lkjmc_store::runtime_adoption::history_count(fixture.database.client_mut(), intent.id)
            .map_err(|error| error.to_string())?
            >= 3
    );
    stop_runtime(&state, "after-intent")?;

    let operation = lkjmc_store::runtime_adoption::allocate(
        fixture.database.client_mut(),
        "after-effect",
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
    let work_dir = data_root.join("after-effect");
    std::fs::create_dir_all(&work_dir).map_err(|error| error.to_string())?;
    let crashed = LocalRuntime::with_data_root(&data_root);
    let effect = crashed.start(
        "after-effect",
        "sleep",
        &["300".to_string()],
        &BTreeMap::new(),
        &fixture.path("logs"),
        Path::new(&work_dir),
        Duration::from_secs(1),
    )?;
    let effect_pid = effect.pid().ok_or("effect crash pid missing")?;
    drop(crashed);
    let restarted = fixture.state();
    let repaired = start_runtime(&restarted, "after-effect")?;
    assert_eq!(repaired.pid(), Some(effect_pid));
    let stale = operation;
    let newer = lkjmc_store::runtime_adoption::allocate(
        fixture.database.client_mut(),
        "after-effect",
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
    start_runtime(&restarted, "after-effect")?;
    stop_runtime(&restarted, "after-effect")?;
    restarted.shutdown_runtime()?;
    assert!(!process::group_exists(intent_pid));
    assert!(!process::group_exists(effect_pid));
    Ok(())
}
