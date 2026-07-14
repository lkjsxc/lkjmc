use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::runtime::{ProcessIdentity, RuntimeAdapter, RuntimeCapabilities, RuntimeObservation};

pub(super) struct Harness {
    pub database: crate::test_database::TestDatabase,
    pub state: AppState,
    root: PathBuf,
}

type Channels = (Harness, mpsc::Receiver<()>, mpsc::Sender<()>);

impl Harness {
    pub fn new(at: BlockAt) -> Result<Option<Channels>, String> {
        let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            return Ok(None);
        };
        let mut database = crate::test_database::migrate(&url)?;
        let root = std::env::temp_dir().join(format!("lkjmc-pool-{}", Uuid::new_v4().simple()));
        let (entered_tx, entered) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let adapter = Arc::new(BlockingAdapter {
            at,
            entered: Mutex::new(Some(entered_tx)),
            release: Mutex::new(release_rx),
        });
        let secret = root.join("config/forwarding.secret");
        std::fs::create_dir_all(secret.parent().ok_or("secret parent missing")?)
            .map_err(|error| error.to_string())?;
        std::fs::write(&secret, "pool-probe-secret").map_err(|error| error.to_string())?;
        let state = AppState::with_config_path(
            Some(database.url().to_string()),
            1,
            root.join("config").to_string_lossy().into(),
            root.join("logs").to_string_lossy().into(),
            root.join("jars").to_string_lossy().into(),
            root.join("data").to_string_lossy().into(),
            None,
            None,
            None,
        )
        .with_runtime(adapter)?;
        lkjmc_store::instance::insert(
            database.client_mut(),
            "pool-probe",
            None,
            "folia",
            "running",
            &json!({"serverPort":25567,"forwardingSecretFile":secret,
                "launch":{"command":"python3","args":[]}}),
        )
        .map_err(|error| error.to_string())?;
        Ok(Some((
            Self {
                database,
                state,
                root,
            },
            entered,
            release,
        )))
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = self.state.shutdown_runtime();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockAt {
    Never,
    Start,
    Status,
}

struct BlockingAdapter {
    at: BlockAt,
    entered: Mutex<Option<mpsc::Sender<()>>>,
    release: Mutex<mpsc::Receiver<()>>,
}

impl BlockingAdapter {
    fn block(&self) -> Result<(), String> {
        if let Some(sender) = self.entered.lock().map_err(|_| "entered lock")?.take() {
            sender.send(()).map_err(|error| error.to_string())?;
            self.release
                .lock()
                .map_err(|_| "release lock")?
                .recv()
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

impl RuntimeAdapter for BlockingAdapter {
    fn name(&self) -> &'static str {
        "blocking-test"
    }
    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            process_identity: true,
            readiness: true,
            storage: true,
            secrets: true,
            configuration: true,
            logs: true,
            recovery: true,
        }
    }
    fn check_capabilities(&self) -> Result<(), String> {
        Ok(())
    }
    fn runtime_start(
        &self,
        _: &str,
        _: &str,
        _: &[String],
        _: &BTreeMap<String, String>,
        _: &str,
        _: &Path,
        _: Duration,
    ) -> Result<RuntimeObservation, String> {
        if self.at == BlockAt::Start {
            self.block()?;
        }
        Err("injected start failure".into())
    }
    fn runtime_stop(&self, _: &str, _: Duration) -> Result<RuntimeObservation, String> {
        Err("unexpected stop".into())
    }
    fn runtime_status(&self, _: &str) -> Result<Option<RuntimeObservation>, String> {
        if self.at == BlockAt::Status {
            self.block()?;
        }
        Ok(None)
    }
    fn runtime_adopt(&self, _: &str, _: ProcessIdentity) -> Result<RuntimeObservation, String> {
        Err("unexpected adopt".into())
    }
    fn runtime_logs(&self, _: &str, _: &str, _: usize) -> Result<Vec<String>, String> {
        Ok(vec![])
    }
    fn runtime_delete(&self, _: &str, _: Duration) -> Result<RuntimeObservation, String> {
        Err("unexpected delete".into())
    }
    fn runtime_shutdown(&self, _: Duration) -> Result<(), String> {
        Ok(())
    }
}
