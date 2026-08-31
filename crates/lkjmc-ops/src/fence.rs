use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{OpsError, Result};
use crate::secure_fs::{
    atomic_write, read_regular, sync_directory, validate_ancestry, MAX_CONTROL_FILE_BYTES,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeploymentFence {
    pub schema_version: u32,
    pub operation_id: Uuid,
    pub from_commit: String,
    pub to_commit: String,
    pub manifest_sha256: String,
    pub state_directory: PathBuf,
    pub backup: PathBuf,
    pub rollback_snapshot: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartPermit {
    pub schema_version: u32,
    pub operation_id: Uuid,
    pub to_commit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FenceCheckResult {
    Unfenced,
    PermittedOnce,
}

impl DeploymentFence {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(OpsError::message("unsupported deployment fence schema"));
        }
        require_hex(&self.from_commit, 40, "fence source commit")?;
        require_hex(&self.to_commit, 40, "fence target commit")?;
        require_hex(&self.manifest_sha256, 64, "fence manifest SHA-256")?;
        crate::secure_fs::require_absolute_safe(&self.state_directory, "fence state directory")?;
        crate::secure_fs::require_absolute_safe(&self.backup, "fence backup path")?;
        if !safe_label(&self.rollback_snapshot) {
            return Err(OpsError::message("unsafe fence rollback snapshot label"));
        }
        Ok(())
    }
}

pub fn write_fence(path: &Path, fence: &DeploymentFence, uid: u32, gid: u32) -> Result<()> {
    fence.validate()?;
    let mut raw = serde_json::to_vec(fence)?;
    raw.push(b'\n');
    atomic_write(path, &raw, 0o600, uid, gid)
}

pub fn write_permit(path: &Path, fence: &DeploymentFence, uid: u32, gid: u32) -> Result<()> {
    fence.validate()?;
    if path.exists() || fs::symlink_metadata(path).is_ok() {
        return Err(OpsError::message("deployment start permit already exists"));
    }
    let permit = StartPermit {
        schema_version: 1,
        operation_id: fence.operation_id,
        to_commit: fence.to_commit.clone(),
    };
    let mut raw = serde_json::to_vec(&permit)?;
    raw.push(b'\n');
    atomic_write(path, &raw, 0o400, uid, gid)
}

pub fn check(
    fence_path: &Path,
    permit_path: &Path,
    trusted_root: &Path,
    expected_uid: u32,
    expected_gid: u32,
) -> Result<FenceCheckResult> {
    validate_ancestry(fence_path, trusted_root, expected_uid)?;
    validate_ancestry(permit_path, trusted_root, expected_uid)?;
    let fence_present = fs::symlink_metadata(fence_path).is_ok();
    let permit_present = fs::symlink_metadata(permit_path).is_ok();
    if !fence_present {
        if permit_present {
            return Err(OpsError::message(
                "deployment start permit exists without a fence",
            ));
        }
        return Ok(FenceCheckResult::Unfenced);
    }
    let fence_raw = read_regular(
        fence_path,
        "deployment fence",
        Some(expected_uid),
        Some(expected_gid),
        Some(0o600),
        MAX_CONTROL_FILE_BYTES,
    )?;
    let fence: DeploymentFence = serde_json::from_slice(&fence_raw)
        .map_err(|error| OpsError::context("invalid deployment fence", error))?;
    fence.validate()?;
    if !permit_present {
        return Err(OpsError::message("deployment fence blocks service start"));
    }
    let permit_raw = read_regular(
        permit_path,
        "deployment start permit",
        Some(expected_uid),
        Some(expected_gid),
        Some(0o400),
        MAX_CONTROL_FILE_BYTES,
    )?;
    let permit: StartPermit = serde_json::from_slice(&permit_raw)
        .map_err(|error| OpsError::context("invalid deployment start permit", error))?;
    if permit.schema_version != 1
        || permit.operation_id != fence.operation_id
        || permit.to_commit != fence.to_commit
    {
        return Err(OpsError::message(
            "deployment start permit differs from the active fence",
        ));
    }
    fs::remove_file(permit_path)
        .map_err(|error| OpsError::context("cannot consume deployment start permit", error))?;
    let parent = permit_path
        .parent()
        .ok_or_else(|| OpsError::message("deployment permit has no parent"))?;
    sync_directory(parent)?;
    Ok(FenceCheckResult::PermittedOnce)
}

pub fn remove_fence(path: &Path, uid: u32, gid: u32) -> Result<()> {
    let _ = read_regular(
        path,
        "deployment fence",
        Some(uid),
        Some(gid),
        Some(0o600),
        MAX_CONTROL_FILE_BYTES,
    )?;
    fs::remove_file(path)
        .map_err(|error| OpsError::context("cannot remove accepted deployment fence", error))?;
    let parent = path
        .parent()
        .ok_or_else(|| OpsError::message("deployment fence has no parent"))?;
    sync_directory(parent)
}

fn require_hex(value: &str, length: usize, label: &str) -> Result<()> {
    if value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(OpsError::message(format!(
            "{label} must be {length} lowercase hexadecimal characters"
        )))
    }
}

fn safe_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        && value.as_bytes()[0].is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use uuid::Uuid;

    use super::*;
    use crate::secure_fs::{effective_gid, effective_uid};

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Result<Self> {
            let path = std::env::temp_dir().join(format!("lkjmc-ops-fence-{}", Uuid::new_v4()));
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
    fn fence_consumes_one_matching_permit_and_rejects_replay() -> Result<()> {
        let root = TestRoot::new()?;
        let uid = effective_uid();
        let gid = effective_gid();
        let fence_path = root.0.join("deployment-fence.json");
        let permit_path = root.0.join("start-permit.json");
        assert_eq!(
            check(&fence_path, &permit_path, &root.0, uid, gid)?,
            FenceCheckResult::Unfenced
        );
        let fence = fixture();
        write_fence(&fence_path, &fence, uid, gid)?;
        write_permit(&permit_path, &fence, uid, gid)?;
        assert_eq!(
            check(&fence_path, &permit_path, &root.0, uid, gid)?,
            FenceCheckResult::PermittedOnce
        );
        assert!(!permit_path.exists());
        let error = check(&fence_path, &permit_path, &root.0, uid, gid)
            .err()
            .ok_or_else(|| OpsError::message("permit replay unexpectedly passed"))?;
        assert!(error.to_string().contains("blocks service start"));
        Ok(())
    }

    #[test]
    fn permit_without_fence_fails_closed() -> Result<()> {
        let root = TestRoot::new()?;
        let uid = effective_uid();
        let gid = effective_gid();
        let fence_path = root.0.join("deployment-fence.json");
        let permit_path = root.0.join("start-permit.json");
        crate::secure_fs::atomic_write(&permit_path, b"{}\n", 0o400, uid, gid)?;
        let error = check(&fence_path, &permit_path, &root.0, uid, gid)
            .err()
            .ok_or_else(|| OpsError::message("orphan permit unexpectedly passed"))?;
        assert!(error.to_string().contains("without a fence"));
        Ok(())
    }

    fn fixture() -> DeploymentFence {
        DeploymentFence {
            schema_version: 1,
            operation_id: Uuid::new_v4(),
            from_commit: "a".repeat(40),
            to_commit: "b".repeat(40),
            manifest_sha256: "c".repeat(64),
            state_directory: PathBuf::from(format!(
                "/var/lib/private/lkjmc-deployments/{}",
                "b".repeat(40)
            )),
            backup: PathBuf::from("/var/backups/lkjmc/pre-update.dump"),
            rollback_snapshot: "pre-update".to_string(),
        }
    }
}
