use std::collections::{BTreeMap, BTreeSet};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use lkjmc_core::config::{InstanceIntegration, LkjmcConfig, ReadinessContract};
use lkjmc_core::id::InstanceId;
use lkjmc_core::instance::{DesiredState, InstanceKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{OpsError, Result};
use crate::secure_fs::{require_absolute_safe, require_directory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceIdentity {
    pub uid: u32,
    pub gid: u32,
}

pub fn service_identity(config: &LkjmcConfig) -> Result<ServiceIdentity> {
    let data_root = Path::new(&config.data_root);
    require_absolute_safe(data_root, "configured data root")?;
    let instances = data_root.join("instances");
    let metadata = require_directory(
        &instances,
        "managed instances root",
        None,
        None,
        Some(0o750),
    )?;
    let identity = ServiceIdentity {
        uid: metadata.uid(),
        gid: metadata.gid(),
    };
    if identity.uid == 0 || identity.gid == 0 {
        return Err(OpsError::message(
            "managed instances root does not identify an unprivileged service principal",
        ));
    }
    Ok(identity)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FleetInstance {
    pub id: InstanceId,
    pub kind: InstanceKind,
    pub desired_state: DesiredState,
    pub integration: InstanceIntegration,
    pub readiness: ReadinessContract,
    pub listener_id: String,
    pub bind_host: String,
    pub port: u16,
    pub asset_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginTarget {
    pub instance_id: InstanceId,
    pub artifact: &'static str,
    pub destination: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialTarget {
    pub instance_id: InstanceId,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EulaTarget {
    pub instance_id: InstanceId,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PersistedInstance {
    pub id: String,
    pub kind: String,
    pub desired_state: String,
}

#[derive(Debug, Clone)]
pub struct FleetSnapshot {
    pub revision: u64,
    pub data_root: PathBuf,
    instances: BTreeMap<String, FleetInstance>,
    velocity_id: String,
}

impl FleetSnapshot {
    pub fn from_config(config: &LkjmcConfig) -> Result<Self> {
        config
            .validate()
            .map_err(|error| OpsError::context("invalid canonical lkjmc configuration", error))?;
        let listeners = config
            .network
            .listeners
            .iter()
            .map(|listener| (listener.id.as_str(), listener))
            .collect::<BTreeMap<_, _>>();
        let mut instances = BTreeMap::new();
        let mut velocity_id = None;
        for configured in &config.network.instances {
            let id = InstanceId::parse(configured.id.clone())
                .map_err(|error| OpsError::context("invalid configured instance ID", error))?;
            let listener = listeners.get(configured.listener.as_str()).ok_or_else(|| {
                OpsError::message(format!(
                    "instance {} references missing listener {}",
                    configured.id, configured.listener
                ))
            })?;
            if configured.kind == InstanceKind::Velocity {
                velocity_id = Some(configured.id.clone());
            }
            let value = FleetInstance {
                id,
                kind: configured.kind,
                desired_state: configured.desired_state,
                integration: configured.integration,
                readiness: configured.readiness,
                listener_id: configured.listener.clone(),
                bind_host: listener.bind_host.clone(),
                port: listener.port,
                asset_ids: configured.asset_ids.clone(),
            };
            if instances.insert(configured.id.clone(), value).is_some() {
                return Err(OpsError::message(format!(
                    "duplicate configured instance: {}",
                    configured.id
                )));
            }
        }
        let velocity_id = velocity_id
            .ok_or_else(|| OpsError::message("configured fleet has no Velocity entrypoint"))?;
        let snapshot = Self {
            revision: config.network.revision,
            data_root: PathBuf::from(&config.data_root),
            instances,
            velocity_id,
        };
        snapshot.validate_supported_readiness()?;
        Ok(snapshot)
    }

    pub fn instances(&self) -> impl Iterator<Item = &FleetInstance> {
        self.instances.values()
    }

    pub fn velocity_entry(&self) -> Result<&FleetInstance> {
        self.instances
            .get(&self.velocity_id)
            .ok_or_else(|| OpsError::message("Velocity entrypoint disappeared from fleet snapshot"))
    }

    pub fn backends(&self) -> impl Iterator<Item = &FleetInstance> {
        self.instances
            .values()
            .filter(|instance| instance.kind != InstanceKind::Velocity)
    }

    pub fn plugin_targets(&self) -> Vec<PluginTarget> {
        self.instances
            .values()
            .filter_map(|instance| {
                instance
                    .integration
                    .plugin_artifact()
                    .map(|artifact| PluginTarget {
                        instance_id: instance.id.clone(),
                        artifact,
                        destination: self
                            .instance_root(instance.id.as_str())
                            .join("plugins")
                            .join(artifact),
                    })
            })
            .collect()
    }

    pub fn credential_targets(&self) -> Vec<CredentialTarget> {
        self.instances
            .values()
            .filter(|instance| instance.integration != InstanceIntegration::None)
            .map(|instance| CredentialTarget {
                instance_id: instance.id.clone(),
                path: self
                    .data_root
                    .join("private/plugin-credentials")
                    .join(format!("{}.secret", instance.id.as_str())),
            })
            .collect()
    }

    pub fn eula_targets(&self) -> Vec<EulaTarget> {
        self.instances
            .values()
            .filter(|instance| instance.kind.requires_minecraft_eula())
            .map(|instance| EulaTarget {
                instance_id: instance.id.clone(),
                path: self.instance_root(instance.id.as_str()).join("eula.txt"),
            })
            .collect()
    }

    pub fn compare_persisted(&self, persisted: &[PersistedInstance]) -> Result<()> {
        let mut observed = BTreeMap::new();
        for row in persisted {
            InstanceId::parse(row.id.clone()).map_err(|error| {
                OpsError::context("persisted fleet contains an invalid instance ID", error)
            })?;
            if observed.insert(row.id.as_str(), row).is_some() {
                return Err(OpsError::message(format!(
                    "persisted fleet duplicates instance {}",
                    row.id
                )));
            }
        }
        let expected_ids = self
            .instances
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let observed_ids = observed.keys().copied().collect::<BTreeSet<_>>();
        if expected_ids != observed_ids {
            let missing = expected_ids.difference(&observed_ids).next().copied();
            let unexpected = observed_ids.difference(&expected_ids).next().copied();
            return Err(OpsError::message(match (missing, unexpected) {
                (Some(id), _) => format!("persisted fleet is missing configured instance {id}"),
                (_, Some(id)) => format!("persisted fleet has unexpected instance {id}"),
                _ => "persisted fleet instance set differs".to_string(),
            }));
        }
        for expected in self.instances.values() {
            let row = observed.get(expected.id.as_str()).ok_or_else(|| {
                OpsError::message(format!(
                    "persisted instance vanished: {}",
                    expected.id.as_str()
                ))
            })?;
            if row.kind != expected.kind.as_str() {
                return Err(OpsError::message(format!(
                    "persisted instance {} kind differs: expected {}, observed {}",
                    expected.id.as_str(),
                    expected.kind.as_str(),
                    row.kind
                )));
            }
            if row.desired_state != expected.desired_state.as_str() {
                return Err(OpsError::message(format!(
                    "persisted instance {} desired state differs: expected {}, observed {}",
                    expected.id.as_str(),
                    expected.desired_state.as_str(),
                    row.desired_state
                )));
            }
        }
        Ok(())
    }

    pub fn validate_status(&self, status: &Value, expected_commit: &str) -> Result<()> {
        if status.get("daemon").and_then(Value::as_str) != Some("running")
            || status
                .pointer("/database/connected")
                .and_then(Value::as_bool)
                != Some(true)
        {
            return Err(OpsError::message(
                "daemon or PostgreSQL status is not ready",
            ));
        }
        if status.pointer("/build/commit").and_then(Value::as_str) != Some(expected_commit)
            || status.pointer("/build/dirty").and_then(Value::as_bool) != Some(false)
        {
            return Err(OpsError::message("daemon build identity differs"));
        }
        if status
            .pointer("/instanceSnapshot/truncated")
            .and_then(Value::as_bool)
            == Some(true)
        {
            return Err(OpsError::message(
                "daemon status truncated the configured fleet",
            ));
        }
        let rows = status
            .get("instances")
            .and_then(Value::as_array)
            .ok_or_else(|| OpsError::message("daemon status has no instance inventory"))?;
        let mut observed = BTreeMap::new();
        for row in rows {
            let id = row
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| OpsError::message("daemon status instance has no string ID"))?;
            if observed.insert(id, row).is_some() {
                return Err(OpsError::message(format!(
                    "daemon status duplicates instance {id}"
                )));
            }
        }
        let expected_ids = self
            .instances
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let observed_ids = observed.keys().copied().collect::<BTreeSet<_>>();
        if expected_ids != observed_ids {
            return Err(OpsError::message(
                "daemon status instance set differs from configured fleet",
            ));
        }
        for instance in self.instances.values() {
            let row = observed.get(instance.id.as_str()).ok_or_else(|| {
                OpsError::message(format!(
                    "status instance vanished: {}",
                    instance.id.as_str()
                ))
            })?;
            if row.get("kind").and_then(Value::as_str) != Some(instance.kind.as_str())
                || row.get("desiredState").and_then(Value::as_str)
                    != Some(instance.desired_state.as_str())
            {
                return Err(OpsError::message(format!(
                    "daemon status identity differs for instance {}",
                    instance.id.as_str()
                )));
            }
            if instance.desired_state.requires_service() {
                if row.get("processHealthy").and_then(Value::as_bool) != Some(true) {
                    return Err(OpsError::message(format!(
                        "required instance {} process is not healthy",
                        instance.id.as_str()
                    )));
                }
                if instance.readiness == ReadinessContract::PluginHeartbeat {
                    validate_plugin_readiness(instance, row)?;
                }
            } else if matches!(
                instance.desired_state,
                DesiredState::Stopped | DesiredState::Suspended
            ) && row.get("processHealthy").and_then(Value::as_bool) == Some(true)
            {
                return Err(OpsError::message(format!(
                    "intentionally inactive instance {} is still running",
                    instance.id.as_str()
                )));
            }
        }
        Ok(())
    }

    pub fn instance_root(&self, id: &str) -> PathBuf {
        self.data_root.join("instances").join(id)
    }

    fn validate_supported_readiness(&self) -> Result<()> {
        for instance in self.instances.values() {
            if instance.desired_state.requires_service()
                && instance.readiness == ReadinessContract::Unsupported
            {
                return Err(OpsError::message(format!(
                    "instance {} requires service but has unsupported readiness",
                    instance.id.as_str()
                )));
            }
        }
        Ok(())
    }
}

fn validate_plugin_readiness(instance: &FleetInstance, row: &Value) -> Result<()> {
    if row.get("ready").and_then(Value::as_bool) != Some(true) {
        return Err(OpsError::message(format!(
            "instance {} plugin readiness is false or unknown",
            instance.id.as_str()
        )));
    }
    for (field, label) in [
        ("readinessAgeSeconds", "plugin readiness"),
        ("proxyRegistrationAgeSeconds", "proxy registration"),
    ] {
        let age = row.get(field).and_then(Value::as_u64).ok_or_else(|| {
            OpsError::message(format!(
                "instance {} {label} age is absent",
                instance.id.as_str()
            ))
        })?;
        if age > 30 {
            return Err(OpsError::message(format!(
                "instance {} {label} is stale",
                instance.id.as_str()
            )));
        }
    }
    if row.get("proxyRegistered").and_then(Value::as_bool) != Some(true) {
        return Err(OpsError::message(format!(
            "instance {} is not registered at Velocity",
            instance.id.as_str()
        )));
    }
    Ok(())
}

pub fn read_config(path: &Path) -> Result<LkjmcConfig> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| OpsError::context("cannot read canonical lkjmc configuration", error))?;
    LkjmcConfig::from_json_str(&raw)
        .map_err(|error| OpsError::context("invalid canonical lkjmc configuration", error))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::{chown, PermissionsExt};

    use serde_json::json;

    use super::*;
    use crate::manifest::sha256_bytes;

    #[test]
    fn one_noncanonical_backend_derives_every_inventory_target() -> Result<()> {
        let config = fixture(&[("quartz-world", "paper", "running")], "edge-gateway")?;
        let fleet = FleetSnapshot::from_config(&config)?;
        assert_eq!(fleet.velocity_entry()?.id.as_str(), "edge-gateway");
        assert_eq!(fleet.backends().count(), 1);
        assert_eq!(fleet.eula_targets().len(), 1);
        assert_eq!(fleet.plugin_targets().len(), 2);
        assert_eq!(fleet.credential_targets().len(), 2);
        assert!(!fleet
            .instances()
            .any(|item| matches!(item.id.as_str(), "proxy" | "hub" | "survival")));
        fleet.compare_persisted(&[
            persisted("edge-gateway", "velocity", "running"),
            persisted("quartz-world", "paper", "running"),
        ])?;
        Ok(())
    }

    #[test]
    fn three_backend_kinds_preserve_stopped_readiness_and_name_neutrality() -> Result<()> {
        let config = fixture(
            &[
                ("alpha-world", "paper", "running"),
                ("beta-world", "folia", "running"),
                ("gamma-world", "purpur", "stopped"),
            ],
            "front-door",
        )?;
        let fleet = FleetSnapshot::from_config(&config)?;
        assert_eq!(fleet.backends().count(), 3);
        assert_eq!(fleet.eula_targets().len(), 3);
        assert_eq!(fleet.plugin_targets().len(), 4);
        assert_eq!(fleet.credential_targets().len(), 4);
        let status = status_fixture(&fleet, "b".repeat(40));
        fleet.validate_status(&status, &"b".repeat(40))?;
        Ok(())
    }

    #[test]
    fn persisted_drift_names_the_affected_instance() -> Result<()> {
        let config = fixture(&[("quartz-world", "paper", "running")], "edge-gateway")?;
        let fleet = FleetSnapshot::from_config(&config)?;
        let error = fleet
            .compare_persisted(&[
                persisted("edge-gateway", "velocity", "running"),
                persisted("quartz-world", "folia", "running"),
            ])
            .err()
            .ok_or_else(|| OpsError::message("persisted drift unexpectedly passed"))?;
        assert!(error.to_string().contains("quartz-world"));
        Ok(())
    }

    #[test]
    fn active_custom_server_fails_unsupported_readiness() -> Result<()> {
        let error = fixture(
            &[("mystery-world", "modded-custom", "running")],
            "edge-gateway",
        )
        .and_then(|config| FleetSnapshot::from_config(&config))
        .err()
        .ok_or_else(|| OpsError::message("unsupported readiness unexpectedly passed"))?;
        assert!(error.to_string().contains("mystery-world"));
        assert!(error.to_string().contains("unsupported readiness"));
        Ok(())
    }

    #[test]
    fn service_identity_is_derived_from_the_managed_instances_root() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "lkjmc-service-identity-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let instances = root.join("instances");
        fs::create_dir_all(&instances)?;
        fs::set_permissions(&instances, fs::Permissions::from_mode(0o750))?;
        let current_uid = crate::secure_fs::effective_uid();
        let current_gid = crate::secure_fs::effective_gid();
        let expected_uid = if current_uid == 0 {
            42_424
        } else {
            current_uid
        };
        let expected_gid = if current_uid == 0 {
            42_424
        } else {
            current_gid
        };
        if current_uid == 0 {
            chown(&instances, Some(expected_uid), Some(expected_gid))?;
        }
        let mut config = fixture(&[("quartz-world", "paper", "running")], "edge-gateway")?;
        config.data_root = root.to_string_lossy().into_owned();
        let identity = service_identity(&config)?;
        assert_eq!(identity.uid, expected_uid);
        assert_eq!(identity.gid, expected_gid);
        fs::set_permissions(&instances, fs::Permissions::from_mode(0o755))?;
        let error = service_identity(&config)
            .err()
            .ok_or_else(|| OpsError::message("broad service root mode unexpectedly passed"))?;
        assert!(error.to_string().contains("mode differs"));
        fs::remove_dir_all(root)?;
        Ok(())
    }

    fn persisted(id: &str, kind: &str, desired_state: &str) -> PersistedInstance {
        PersistedInstance {
            id: id.to_string(),
            kind: kind.to_string(),
            desired_state: desired_state.to_string(),
        }
    }

    fn fixture(backends: &[(&str, &str, &str)], velocity: &str) -> Result<LkjmcConfig> {
        let mut instances = Vec::new();
        let mut listeners = Vec::new();
        let mut assets = Vec::new();
        let velocity_asset = "entry-asset";
        instances.push(json!({
            "id": velocity,
            "owner": "lkjmc-daemon",
            "kind": "velocity",
            "desiredState": "running",
            "integration": "velocity",
            "readiness": "velocity-status",
            "listener": format!("{velocity}-java"),
            "memoryMb": 512,
            "assetIds": [velocity_asset]
        }));
        listeners.push(json!({
            "id": format!("{velocity}-java"),
            "protocol": "java-tcp",
            "bindHost": "127.0.0.1",
            "port": 25565,
            "publicHosts": []
        }));
        assets.push(asset(velocity_asset, "velocity"));
        for (index, (id, kind, desired_state)) in backends.iter().enumerate() {
            let custom = matches!(*kind, "vanilla-custom" | "modded-custom");
            let asset_id = format!("asset-{id}");
            instances.push(json!({
                "id": id,
                "owner": "lkjmc-daemon",
                "kind": kind,
                "desiredState": desired_state,
                "integration": if custom { "none" } else { "paper-compatible" },
                "readiness": if custom { "unsupported" } else { "plugin-heartbeat" },
                "listener": format!("{id}-java"),
                "memoryMb": 1024,
                "assetIds": [asset_id]
            }));
            listeners.push(json!({
                "id": format!("{id}-java"),
                "protocol": "java-tcp",
                "bindHost": "127.0.0.1",
                "port": 25566 + index,
                "publicHosts": []
            }));
            assets.push(asset(&asset_id, id));
        }
        let fallback_ids = backends
            .iter()
            .skip(1)
            .map(|(id, _, _)| *id)
            .collect::<Vec<_>>();
        let target = backends
            .first()
            .map(|(id, _, _)| *id)
            .ok_or_else(|| OpsError::message("fixture needs a backend"))?;
        let value = json!({
            "installRoot": "/opt/lkjmc",
            "configRoot": "/etc/lkjmc",
            "dataRoot": "/var/lib/lkjmc",
            "logRoot": "/var/log/lkjmc",
            "socketPath": "/run/lkjmc/daemon.sock",
            "database": {"host":"127.0.0.1","port":5432,"database":"lkjmc","user":"lkjmc","secretFile":"/etc/lkjmc/database.secret"},
            "network": {
                "revision": 7,
                "instances": instances,
                "routes": [{"id":"primary-route","listener":format!("{velocity}-java"),"target":target,"fallbacks":fallback_ids}],
                "listeners": listeners,
                "auth": {"onlineMode":true},
                "forwarding": {"mode":"modern","secretFile":"/etc/lkjmc/forwarding.secret"},
                "assets": assets,
                "capabilities": {"runtime":"local-process","mountedConfig":true,"mountedSecrets":true,"mountedAssets":true}
            },
            "jars": {"root":"/opt/lkjmc/jars","defaultChannel":"stable","userAgent":"lkjmc (+https://github.com/lkjsxc/lkjmc)"},
            "runtime": {"adapter":"local-process","defaultJavaMemoryMb":1024,"proxyJavaMemoryMb":512,"stopTimeoutSeconds":30,"portRangeStart":25565,"portRangeEnd":25665}
        });
        LkjmcConfig::from_json_str(&value.to_string())
            .map_err(|error| OpsError::context("fixture config failed", error))
    }

    fn asset(id: &str, seed: &str) -> Value {
        json!({
            "id": id,
            "kind": "server",
            "path": format!("/opt/lkjmc/assets/{id}.jar"),
            "sha256": sha256_bytes(seed.as_bytes()),
            "required": true
        })
    }

    fn status_fixture(fleet: &FleetSnapshot, commit: String) -> Value {
        let instances = fleet
            .instances()
            .map(|instance| {
                let running = instance.desired_state.requires_service();
                let plugin = instance.readiness == ReadinessContract::PluginHeartbeat;
                json!({
                    "id": instance.id.as_str(),
                    "kind": instance.kind.as_str(),
                    "desiredState": instance.desired_state.as_str(),
                    "processHealthy": running,
                    "ready": if plugin { Value::Bool(true) } else { Value::Null },
                    "readinessAgeSeconds": if plugin { Value::from(2) } else { Value::Null },
                    "proxyRegistered": if plugin { Value::Bool(true) } else { Value::Null },
                    "proxyRegistrationAgeSeconds": if plugin { Value::from(2) } else { Value::Null }
                })
            })
            .collect::<Vec<_>>();
        json!({
            "daemon": "running",
            "database": {"connected": true},
            "build": {"commit": commit, "dirty": false},
            "instanceSnapshot": {"truncated": false},
            "instances": instances
        })
    }
}
