#[path = "fixture_config.rs"]
mod fixture_config;
#[path = "fixture_repair.rs"]
mod fixture_repair;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;

use fixture_config::{build_state, insert_instance, write_config};
use lkjmc_core::config::LkjmcConfig;
use lkjmc_store::network_intent::ApplyAttempt;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;
use crate::runtime::RuntimeObservation;

pub(super) const PRIMARY_BACKEND_ID: &str = "quartz-world";
pub(super) const SECONDARY_BACKEND_ID: &str = "ember-realm";
pub(super) const ENTRY_ID: &str = "edge-gateway";
pub(super) const ENTRY_LISTENER_ID: &str = "edge-java";

pub(super) struct Fixture {
    pub database: crate::test_database::TestDatabase,
    pub root: PathBuf,
    pub config: LkjmcConfig,
    pub state: AppState,
}

impl Fixture {
    pub fn new(proxy_command: &str) -> Result<Option<Self>, String> {
        let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
            return Ok(None);
        };
        let mut database = crate::test_database::migrate(&url)?;
        let root =
            std::env::temp_dir().join(format!("lkjmc-network-probe-{}", Uuid::new_v4().simple()));
        std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let config = write_config(&root, proxy_command == "python3")?;
        materialize_eula(&config)?;
        let state = build_state(&root, database.url().to_string())?;
        lkjmc_store::network_intent::record_desired(
            database.client_mut(),
            1,
            &config.network.digest(),
            &serde_json::to_value(&config.network).map_err(|error| error.to_string())?,
            "network-probe-seed",
        )
        .map_err(|error| error.to_string())?;
        insert_instance(
            database.client_mut(),
            &config,
            PRIMARY_BACKEND_ID,
            "folia",
            "python3",
        )?;
        insert_instance(
            database.client_mut(),
            &config,
            ENTRY_ID,
            "velocity",
            proxy_command,
        )?;
        insert_instance(
            database.client_mut(),
            &config,
            SECONDARY_BACKEND_ID,
            "folia",
            "python3",
        )?;
        Ok(Some(Self {
            database,
            root,
            config,
            state,
        }))
    }

    pub fn seed_newer_folia_decoy(&mut self) -> Result<Uuid, String> {
        super::super::network_plan::register_assets(&self.state, &self.config)?;
        std::thread::sleep(std::time::Duration::from_millis(2));
        let path = self.root.join("assets/newer-folia-decoy.jar");
        let bytes = b"not the configured folia jar";
        std::fs::write(&path, bytes).map_err(|error| error.to_string())?;
        let id = Uuid::new_v4();
        lkjmc_store::jar::insert(
            self.database.client_mut(),
            lkjmc_store::jar::NewJarAsset {
                id,
                kind: "folia",
                project: "folia",
                channel: "decoy",
                name: "newer-folia-decoy.jar",
                path: path.to_string_lossy().as_ref(),
                sha256: &format!("{:x}", Sha256::digest(bytes)),
                size_bytes: i64::try_from(bytes.len()).map_err(|error| error.to_string())?,
                source: "test-decoy",
            },
        )
        .map_err(|error| error.to_string())?;
        Ok(id)
    }

    pub fn bind_instance_to_jar(&mut self, instance_id: &str, jar_id: Uuid) -> Result<(), String> {
        let mut config = lkjmc_store::instance::config(self.database.client_mut(), instance_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("instance config missing: {instance_id}"))?;
        config["jarAssetId"] = json!(jar_id.to_string());
        lkjmc_store::instance::update_config(self.database.client_mut(), instance_id, &config)
            .map_err(|error| error.to_string())?;
        lkjmc_store::instance::set_jar_asset(self.database.client_mut(), instance_id, jar_id)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn make_instance_config_legacy(&mut self, instance_id: &str) -> Result<(), String> {
        let mut config = lkjmc_store::instance::config(self.database.client_mut(), instance_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("instance config missing: {instance_id}"))?;
        config
            .as_object_mut()
            .ok_or("instance config is not an object")?
            .remove("configSchemaVersion");
        config["env"] = json!({
            "LKJMC_INSTANCE_ID": instance_id,
            "LKJMC_DAEMON_HTTP_URL": "http://127.0.0.1:8765",
            "LKJMC_SERVER_IMPLEMENTATION": if instance_id == ENTRY_ID { "velocity" } else { "folia" }
        });
        lkjmc_store::instance::update_config(self.database.client_mut(), instance_id, &config)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    pub fn instance_config(&mut self, instance_id: &str) -> Result<Value, String> {
        lkjmc_store::instance::config(self.database.client_mut(), instance_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("instance config missing: {instance_id}"))
    }

    pub fn selected_jar_path(&mut self, id: &str) -> Result<String, String> {
        self.database
            .client_mut()
            .query_one(
                "select j.path from instances i join jar_assets j on j.id=i.jar_asset_id where i.id=$1",
                &[&id],
            )
            .map(|row| row.get(0))
            .map_err(|error| error.to_string())
    }

    pub fn repair_proxy(&mut self) -> Result<(), String> {
        fixture_repair::repair_proxy(self)
    }

    pub fn restarted_state(&self) -> Result<AppState, String> {
        build_state(&self.root, self.database.url().to_string())
    }

    pub fn seed_attempt(&mut self, correlation: &str, phase: &str) -> Result<Uuid, String> {
        let desired = lkjmc_store::network_intent::latest_desired(self.database.client_mut())
            .map_err(|error| error.to_string())?
            .ok_or("desired network missing")?;
        let attempt = lkjmc_store::network_intent::create_attempt(
            self.database.client_mut(),
            desired.revision,
            correlation,
        )
        .map_err(|error| error.to_string())?;
        lkjmc_store::network_intent::mark_applying(self.database.client_mut(), attempt.id)
            .map_err(|error| error.to_string())?;
        if phase != "none" {
            lkjmc_store::network_intent::mark_effect_phase(
                self.database.client_mut(),
                attempt.id,
                phase,
            )
            .map_err(|error| error.to_string())?;
        }
        Ok(attempt.id)
    }

    pub fn attempt(&mut self, id: Uuid) -> Result<ApplyAttempt, String> {
        lkjmc_store::network_intent::attempt(self.database.client_mut(), id)
            .map_err(|error| error.to_string())?
            .ok_or("network attempt missing".to_string())
    }

    pub fn attempts_for(&mut self, correlation: &str) -> Result<Vec<ApplyAttempt>, String> {
        lkjmc_store::network_intent::attempts_for_correlation(
            self.database.client_mut(),
            correlation,
        )
        .map_err(|error| error.to_string())
    }

    pub fn start_proxy(&mut self) -> Result<RuntimeObservation, String> {
        self.prepare_proxy_shape()?;
        crate::support::instance_helpers::start_runtime(&self.state, ENTRY_ID)
    }

    fn prepare_proxy_shape(&mut self) -> Result<(), String> {
        super::super::network_plan::register_assets(&self.state, &self.config)?;
        let inspection = super::super::super::network_state::inspect(&self.state)?;
        let request = super::request("fixture-prepare-proxy")?;
        for effect in super::super::network_plan::effects(&self.config, &inspection)? {
            let proxy_effect = match &effect {
                super::super::network_plan::NetworkEffect::ReconcileInstance { id, .. }
                | super::super::network_plan::NetworkEffect::RenderInstance { id } => {
                    id.as_str() == ENTRY_ID
                }
                _ => false,
            };
            if proxy_effect {
                super::super::effects::apply_effect(&self.state, &request, &effect)?;
            }
        }
        Ok(())
    }

    pub fn render_proxy(&mut self) -> Result<(), String> {
        self.prepare_proxy_shape()?;
        let row = lkjmc_store::instance::get(self.database.client_mut(), ENTRY_ID)
            .map_err(|error| error.to_string())?
            .ok_or("proxy missing")?;
        let config = lkjmc_store::instance::config(self.database.client_mut(), ENTRY_ID)
            .map_err(|error| error.to_string())?
            .ok_or("proxy config missing")?;
        crate::templates::render_instance(&self.state, ENTRY_ID, &row.kind, &config).map(|_| ())
    }

    pub fn set_proxy_stopped(&self) -> Result<(), String> {
        let path = self.root.join("lkjmc.json");
        let mut value: Value =
            serde_json::from_slice(&std::fs::read(&path).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        value["network"]["revision"] = json!(2);
        value["network"]["instances"][1]["desiredState"] = json!("stopped");
        let text = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
        LkjmcConfig::from_json_str(&text).map_err(|error| error.to_string())?;
        std::fs::write(path, text).map_err(|error| error.to_string())
    }

    pub fn pid(&mut self, id: &str) -> Result<u32, String> {
        lkjmc_store::instance::get(self.database.client_mut(), id)
            .map_err(|error| error.to_string())?
            .and_then(|row| row.pid)
            .and_then(|pid| u32::try_from(pid).ok())
            .ok_or_else(|| format!("instance pid missing: {id}"))
    }

    pub fn runtime_history_count(&mut self, id: &str) -> Result<i64, String> {
        self.database
            .client_mut()
            .query_one(
                "select count(*) from runtime_effect_workflows where instance_id = $1",
                &[&id],
            )
            .map(|row| row.get(0))
            .map_err(|error| error.to_string())
    }

    pub fn tracked_pids(&mut self) -> Result<Vec<u32>, String> {
        Ok(lkjmc_store::instance::list(self.database.client_mut())
            .map_err(|error| error.to_string())?
            .into_iter()
            .filter_map(|row| row.pid.and_then(|pid| u32::try_from(pid).ok()))
            .collect())
    }

    pub fn cleanup(&self) {
        let _ = crate::support::instance_helpers::stop_runtime(&self.state, ENTRY_ID);
        let _ = crate::support::instance_helpers::stop_runtime(&self.state, PRIMARY_BACKEND_ID);
        let _ = crate::support::instance_helpers::stop_runtime(&self.state, SECONDARY_BACKEND_ID);
        let _ = self.state.shutdown_runtime();
    }
}

fn materialize_eula(config: &LkjmcConfig) -> Result<(), String> {
    let config_root = PathBuf::from(&config.config_root);
    let instances_root = PathBuf::from(&config.data_root).join("instances");
    for path in [&config_root, &instances_root] {
        fs::create_dir_all(path).map_err(|error| error.to_string())?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o750))
            .map_err(|error| error.to_string())?;
    }
    for id in [PRIMARY_BACKEND_ID, SECONDARY_BACKEND_ID, ENTRY_ID] {
        let path = instances_root.join(id);
        fs::create_dir(&path).map_err(|error| error.to_string())?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o750))
            .map_err(|error| error.to_string())?;
    }
    let metadata = fs::symlink_metadata(&instances_root).map_err(|error| error.to_string())?;
    let uid = metadata.uid();
    let gid = metadata.gid();
    let policy = config_root.join("minecraft-eula.accepted");
    lkjmc_ops::eula::create_policy(&policy, uid, gid).map_err(|error| error.to_string())?;
    let fleet =
        lkjmc_ops::fleet::FleetSnapshot::from_config(config).map_err(|error| error.to_string())?;
    let _ = lkjmc_ops::eula::materialize(&fleet, &policy, uid, uid, gid)
        .map_err(|error| error.to_string())?;
    Ok(())
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.cleanup();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
