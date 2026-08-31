use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use serde::Serialize;

use crate::error::{OpsError, Result};
use crate::fleet::FleetSnapshot;
use crate::manifest::sha256_bytes;
use crate::secure_fs::{
    atomic_write, read_regular, require_absolute_safe, require_directory, require_regular,
    MAX_CONTROL_FILE_BYTES,
};

pub const POLICY_BYTES: &[u8] = b"schemaVersion=1\naccepted=true\n";
pub const EULA_BYTES: &[u8] = b"eula=true\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EulaReceipt {
    pub schema_version: u32,
    pub policy_sha256: String,
    pub materialized_instances: Vec<String>,
    pub unchanged_instances: Vec<String>,
}

pub fn create_policy(path: &Path, expected_uid: u32, expected_gid: u32) -> Result<bool> {
    require_absolute_safe(path, "Minecraft EULA policy path")?;
    let parent = path
        .parent()
        .ok_or_else(|| OpsError::message("Minecraft EULA policy path has no parent"))?;
    require_directory(
        parent,
        "Minecraft EULA policy directory",
        Some(expected_uid),
        Some(expected_gid),
        None,
    )?;
    if let Ok(existing) = read_regular(
        path,
        "Minecraft EULA policy",
        Some(expected_uid),
        Some(expected_gid),
        Some(0o440),
        MAX_CONTROL_FILE_BYTES,
    ) {
        if existing == POLICY_BYTES {
            return Ok(false);
        }
    }
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if !metadata.file_type().is_file()
            || metadata.file_type().is_symlink()
            || metadata.uid() != expected_uid
            || metadata.gid() != expected_gid
            || metadata.permissions().mode() & 0o022 != 0
        {
            return Err(OpsError::message(
                "refusing to replace an unsafe Minecraft EULA policy",
            ));
        }
    }
    atomic_write(path, POLICY_BYTES, 0o440, expected_uid, expected_gid)?;
    verify_policy(path, expected_uid, expected_gid)?;
    Ok(true)
}

pub fn verify_policy(path: &Path, expected_uid: u32, expected_gid: u32) -> Result<String> {
    require_absolute_safe(path, "Minecraft EULA policy path")?;
    let raw = read_regular(
        path,
        "Minecraft EULA policy",
        Some(expected_uid),
        Some(expected_gid),
        Some(0o440),
        MAX_CONTROL_FILE_BYTES,
    )?;
    if raw != POLICY_BYTES {
        return Err(OpsError::message(
            "Minecraft EULA policy schema or acceptance differs",
        ));
    }
    Ok(sha256_bytes(&raw))
}

pub fn materialize(
    fleet: &FleetSnapshot,
    policy_path: &Path,
    policy_uid: u32,
    service_uid: u32,
    service_gid: u32,
) -> Result<EulaReceipt> {
    let policy_sha256 = verify_policy(policy_path, policy_uid, service_gid)?;
    require_absolute_safe(&fleet.data_root, "fleet data root")?;
    let instances_root = fleet.data_root.join("instances");
    require_directory(
        &instances_root,
        "managed instances root",
        Some(service_uid),
        Some(service_gid),
        Some(0o750),
    )?;
    let mut materialized_instances = Vec::new();
    let mut unchanged_instances = Vec::new();
    for target in fleet.eula_targets() {
        let parent = target
            .path
            .parent()
            .ok_or_else(|| OpsError::message("EULA target has no instance directory"))?;
        if parent.parent() != Some(instances_root.as_path()) {
            return Err(OpsError::message(format!(
                "EULA target for {} escapes the managed instances root",
                target.instance_id.as_str()
            )));
        }
        require_directory(
            parent,
            "managed instance directory",
            Some(service_uid),
            Some(service_gid),
            Some(0o750),
        )?;
        let existing = read_regular(
            &target.path,
            "managed instance EULA file",
            Some(policy_uid),
            Some(service_gid),
            Some(0o640),
            MAX_CONTROL_FILE_BYTES,
        );
        match existing {
            Ok(raw) if raw == EULA_BYTES => {
                unchanged_instances.push(target.instance_id.as_str().to_string());
            }
            Ok(_) => {
                atomic_write(&target.path, EULA_BYTES, 0o640, policy_uid, service_gid)?;
                independently_verify_eula(&target.path, policy_uid, service_gid)?;
                materialized_instances.push(target.instance_id.as_str().to_string());
            }
            Err(error) => {
                if target.path.exists() || fs::symlink_metadata(&target.path).is_ok() {
                    return Err(OpsError::message(format!(
                        "unsafe EULA state for instance {}: {error}",
                        target.instance_id.as_str()
                    )));
                }
                atomic_write(&target.path, EULA_BYTES, 0o640, policy_uid, service_gid)?;
                independently_verify_eula(&target.path, policy_uid, service_gid)?;
                materialized_instances.push(target.instance_id.as_str().to_string());
            }
        }
    }
    Ok(EulaReceipt {
        schema_version: 1,
        policy_sha256,
        materialized_instances,
        unchanged_instances,
    })
}

fn independently_verify_eula(path: &Path, uid: u32, gid: u32) -> Result<()> {
    let metadata = require_regular(
        path,
        "materialized Minecraft EULA file",
        Some(uid),
        Some(gid),
        Some(0o640),
        MAX_CONTROL_FILE_BYTES,
    )?;
    if metadata.len() != EULA_BYTES.len() as u64 {
        return Err(OpsError::message(
            "materialized Minecraft EULA file size differs",
        ));
    }
    let raw = fs::read(path)
        .map_err(|error| OpsError::context("cannot reread materialized EULA file", error))?;
    if raw != EULA_BYTES {
        return Err(OpsError::message(
            "materialized Minecraft EULA file contents differ",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{chown, symlink, PermissionsExt};
    use std::path::PathBuf;

    use lkjmc_core::config::LkjmcConfig;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::fleet::FleetSnapshot;
    use crate::secure_fs::{effective_gid, effective_uid};

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Result<Self> {
            let path = std::env::temp_dir().join(format!("lkjmc-ops-eula-{}", Uuid::new_v4()));
            fs::create_dir(&path)?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
            Ok(Self(path))
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn policy_materializes_each_noncanonical_backend_once() -> Result<()> {
        let test = TestRoot::new()?;
        let uid = effective_uid();
        let gid = effective_gid();
        let config_root = test.0.join("config");
        let data_root = test.0.join("data");
        make_directory(&config_root, uid, gid)?;
        make_directory(&data_root, uid, gid)?;
        make_directory(&data_root.join("instances"), uid, gid)?;
        for id in ["alpha-world", "beta-world", "front-door"] {
            make_directory(&data_root.join("instances").join(id), uid, gid)?;
        }
        let policy = config_root.join("minecraft-eula.accepted");
        assert!(create_policy(&policy, uid, gid)?);
        assert!(!create_policy(&policy, uid, gid)?);
        let fleet = FleetSnapshot::from_config(&fixture(&data_root)?)?;
        let first = materialize(&fleet, &policy, uid, uid, gid)?;
        assert_eq!(first.materialized_instances, ["alpha-world", "beta-world"]);
        assert!(first.unchanged_instances.is_empty());
        assert_eq!(
            fs::read(data_root.join("instances/alpha-world/eula.txt"))?,
            EULA_BYTES
        );
        assert!(!data_root.join("instances/front-door/eula.txt").exists());
        let second = materialize(&fleet, &policy, uid, uid, gid)?;
        assert!(second.materialized_instances.is_empty());
        assert_eq!(second.unchanged_instances, ["alpha-world", "beta-world"]);
        Ok(())
    }

    #[test]
    fn symlink_eula_fails_without_mutating_target() -> Result<()> {
        let test = TestRoot::new()?;
        let uid = effective_uid();
        let gid = effective_gid();
        let config_root = test.0.join("config");
        let data_root = test.0.join("data");
        make_directory(&config_root, uid, gid)?;
        make_directory(&data_root, uid, gid)?;
        make_directory(&data_root.join("instances"), uid, gid)?;
        for id in ["alpha-world", "beta-world", "front-door"] {
            make_directory(&data_root.join("instances").join(id), uid, gid)?;
        }
        let policy = config_root.join("minecraft-eula.accepted");
        create_policy(&policy, uid, gid)?;
        let unrelated = test.0.join("unrelated");
        fs::write(&unrelated, b"preserve\n")?;
        symlink(&unrelated, data_root.join("instances/alpha-world/eula.txt"))?;
        let fleet = FleetSnapshot::from_config(&fixture(&data_root)?)?;
        let error = materialize(&fleet, &policy, uid, uid, gid)
            .err()
            .ok_or_else(|| OpsError::message("symlink EULA unexpectedly passed"))?;
        assert!(error.to_string().contains("alpha-world"));
        assert_eq!(fs::read(&unrelated)?, b"preserve\n");
        Ok(())
    }

    fn make_directory(path: &Path, uid: u32, gid: u32) -> Result<()> {
        fs::create_dir(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o750))?;
        chown(path, Some(uid), Some(gid))?;
        Ok(())
    }

    fn fixture(data_root: &Path) -> Result<LkjmcConfig> {
        let digest_a = crate::manifest::sha256_bytes(b"a");
        let digest_b = crate::manifest::sha256_bytes(b"b");
        let digest_c = crate::manifest::sha256_bytes(b"c");
        let value = json!({
            "installRoot": "/opt/lkjmc", "configRoot": "/etc/lkjmc",
            "dataRoot": data_root, "logRoot": "/var/log/lkjmc",
            "socketPath": "/run/lkjmc/daemon.sock",
            "database": {"host":"127.0.0.1","port":5432,"database":"lkjmc","user":"lkjmc","secretFile":"/etc/lkjmc/database.secret"},
            "network": {
                "revision": 3,
                "instances": [
                    {"id":"front-door","owner":"lkjmc-daemon","kind":"velocity","desiredState":"running","integration":"velocity","readiness":"velocity-status","listener":"front-java","memoryMb":512,"assetIds":["velocity-bin"]},
                    {"id":"alpha-world","owner":"lkjmc-daemon","kind":"paper","desiredState":"running","integration":"paper-compatible","readiness":"plugin-heartbeat","listener":"alpha-java","memoryMb":1024,"assetIds":["paper-bin"]},
                    {"id":"beta-world","owner":"lkjmc-daemon","kind":"folia","desiredState":"stopped","integration":"paper-compatible","readiness":"plugin-heartbeat","listener":"beta-java","memoryMb":1024,"assetIds":["folia-bin"]}
                ],
                "routes": [{"id":"primary","listener":"front-java","target":"alpha-world","fallbacks":["beta-world"]}],
                "listeners": [
                    {"id":"front-java","protocol":"java-tcp","bindHost":"127.0.0.1","port":25565,"publicHosts":[]},
                    {"id":"alpha-java","protocol":"java-tcp","bindHost":"127.0.0.1","port":25566,"publicHosts":[]},
                    {"id":"beta-java","protocol":"java-tcp","bindHost":"127.0.0.1","port":25567,"publicHosts":[]}
                ],
                "auth":{"onlineMode":true},
                "forwarding":{"mode":"modern","secretFile":"/etc/lkjmc/forwarding.secret"},
                "assets": [
                    {"id":"velocity-bin","kind":"server","path":"/opt/lkjmc/assets/velocity.jar","sha256":digest_a,"required":true},
                    {"id":"paper-bin","kind":"server","path":"/opt/lkjmc/assets/paper.jar","sha256":digest_b,"required":true},
                    {"id":"folia-bin","kind":"server","path":"/opt/lkjmc/assets/folia.jar","sha256":digest_c,"required":true}
                ],
                "capabilities":{"runtime":"local-process","mountedConfig":true,"mountedSecrets":true,"mountedAssets":true}
            },
            "jars":{"root":"/opt/lkjmc/jars","defaultChannel":"stable","userAgent":"lkjmc (+https://github.com/lkjsxc/lkjmc)"},
            "runtime":{"adapter":"local-process","defaultJavaMemoryMb":1024,"proxyJavaMemoryMb":512,"stopTimeoutSeconds":30,"portRangeStart":25565,"portRangeEnd":25665}
        });
        LkjmcConfig::from_json_str(&value.to_string())
            .map_err(|error| OpsError::context("EULA fixture config failed", error))
    }
}
