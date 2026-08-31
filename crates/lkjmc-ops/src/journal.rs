use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{OpsError, Result};
use crate::secure_fs::{
    atomic_write, read_regular, require_absolute_safe, require_directory, sync_directory,
    validate_ancestry, MAX_CONTROL_FILE_BYTES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationPhase {
    Preflight,
    NoOp,
    BackupVerified,
    Fenced,
    ServiceStopped,
    ArtifactsStaged,
    MigrationClassified,
    Activated,
    ServiceStarting,
    PostStartVerifying,
    Accepted,
    Abandoned,
    RolledBack,
    RestoreRequired,
    RecoveryBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MigrationIdentity {
    pub version: u32,
    pub name: String,
    pub checksum: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupClosure {
    pub dump: PathBuf,
    pub manifest: PathBuf,
    pub metadata: PathBuf,
    pub checksums: PathBuf,
    pub dump_sha256: String,
    pub manifest_sha256: String,
    pub metadata_sha256: String,
    pub source_commit: String,
    pub schema_identity: String,
    pub migration_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeploymentJournal {
    pub schema_version: u32,
    pub operation_id: Uuid,
    pub source_commit: String,
    pub source_manifest_sha256: String,
    pub target_commit: String,
    pub manifest_sha256: String,
    pub state_directory: PathBuf,
    pub prior_release_root: PathBuf,
    pub prior_unit_sha256: String,
    pub prior_fence_dropin_sha256: String,
    pub prior_plugins: BTreeMap<String, String>,
    pub migration_before: Vec<MigrationIdentity>,
    pub migration_after: Option<Vec<MigrationIdentity>>,
    pub backup_path: PathBuf,
    pub backup: Option<BackupClosure>,
    pub rollback_snapshot: Option<String>,
    pub phase: OperationPhase,
    pub first_failure: Option<String>,
    pub recovery_decision: Option<RecoveryDecision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecoveryDecision {
    SafeBinaryRollback,
    DataAwareRestoreRequired,
    RecoveryBlocked,
}

impl DeploymentJournal {
    pub fn transition(&mut self, next: OperationPhase) -> Result<()> {
        if !allowed_transition(self.phase, next) {
            return Err(OpsError::message(format!(
                "invalid deployment phase transition: {:?} -> {:?}",
                self.phase, next
            )));
        }
        self.phase = next;
        Ok(())
    }

    pub fn record_failure(&mut self, failure: impl Into<String>) {
        if self.first_failure.is_none() {
            self.first_failure = Some(bounded_failure(failure.into()));
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(OpsError::message("unsupported deployment journal schema"));
        }
        require_hex(&self.source_commit, 40, "journal source commit")?;
        require_hex(
            &self.source_manifest_sha256,
            64,
            "journal source manifest SHA-256",
        )?;
        require_hex(&self.target_commit, 40, "journal target commit")?;
        require_hex(&self.manifest_sha256, 64, "journal manifest SHA-256")?;
        require_absolute_safe(&self.state_directory, "deployment state directory")?;
        require_hex(&self.prior_unit_sha256, 64, "journal unit SHA-256")?;
        require_hex(
            &self.prior_fence_dropin_sha256,
            64,
            "journal fence drop-in SHA-256",
        )?;
        require_absolute_safe(&self.prior_release_root, "prior release root")?;
        if self.prior_plugins.len() > 64 {
            return Err(OpsError::message(
                "journal plugin inventory exceeds 64 items",
            ));
        }
        for (id, digest) in &self.prior_plugins {
            lkjmc_core::id::InstanceId::parse(id.clone())
                .map_err(|error| OpsError::context("invalid journal plugin instance", error))?;
            require_hex(digest, 64, "journal plugin SHA-256")?;
        }
        validate_migrations(&self.migration_before)?;
        if let Some(after) = &self.migration_after {
            validate_migrations(after)?;
        }
        require_absolute_safe(&self.backup_path, "planned backup path")?;
        if let Some(backup) = &self.backup {
            validate_backup(backup, &self.source_commit)?;
            if backup.dump.parent() != Some(self.backup_path.as_path()) {
                return Err(OpsError::message(
                    "verified backup closure differs from the planned backup path",
                ));
            }
        } else if !matches!(
            self.phase,
            OperationPhase::Preflight | OperationPhase::NoOp | OperationPhase::Abandoned
        ) {
            return Err(OpsError::message(
                "durable deployment phase requires a verified backup closure",
            ));
        }
        if self
            .rollback_snapshot
            .as_deref()
            .is_some_and(|value| !safe_label(value))
        {
            return Err(OpsError::message("unsafe rollback snapshot label"));
        }
        if self
            .first_failure
            .as_deref()
            .is_some_and(|failure| failure.len() > 4096 || failure.contains('\n'))
        {
            return Err(OpsError::message("journal failure receipt is unbounded"));
        }
        Ok(())
    }
}

fn validate_backup(backup: &BackupClosure, source_commit: &str) -> Result<()> {
    for (path, label) in [
        (&backup.dump, "backup dump"),
        (&backup.manifest, "backup manifest"),
        (&backup.metadata, "backup metadata"),
        (&backup.checksums, "backup checksums"),
    ] {
        require_absolute_safe(path, label)?;
    }
    let parent = backup.dump.parent();
    if parent.is_none()
        || backup.manifest.parent() != parent
        || backup.metadata.parent() != parent
        || backup.checksums.parent() != parent
    {
        return Err(OpsError::message(
            "backup closure members do not share one directory",
        ));
    }
    for (digest, label) in [
        (&backup.dump_sha256, "backup dump SHA-256"),
        (&backup.manifest_sha256, "backup manifest SHA-256"),
        (&backup.metadata_sha256, "backup metadata SHA-256"),
        (&backup.schema_identity, "backup schema identity"),
        (&backup.migration_identity, "backup migration identity"),
    ] {
        require_hex(digest, 64, label)?;
    }
    if backup.source_commit != source_commit {
        return Err(OpsError::message(
            "backup closure source commit differs from the deployment journal",
        ));
    }
    Ok(())
}

pub fn classify_recovery(
    before: &[MigrationIdentity],
    observed: Option<&[MigrationIdentity]>,
) -> RecoveryDecision {
    match observed {
        Some(value) if value == before => RecoveryDecision::SafeBinaryRollback,
        Some(_) => RecoveryDecision::DataAwareRestoreRequired,
        None => RecoveryDecision::RecoveryBlocked,
    }
}

pub fn write_journal(path: &Path, journal: &DeploymentJournal, uid: u32, gid: u32) -> Result<()> {
    journal.validate()?;
    let mut raw = serde_json::to_vec(journal)?;
    raw.push(b'\n');
    atomic_write(path, &raw, 0o600, uid, gid)
}

pub fn read_journal(path: &Path, uid: u32, gid: u32) -> Result<DeploymentJournal> {
    let raw = read_regular(
        path,
        "deployment journal",
        Some(uid),
        Some(gid),
        Some(0o600),
        MAX_CONTROL_FILE_BYTES,
    )?;
    let journal: DeploymentJournal = serde_json::from_slice(&raw)
        .map_err(|error| OpsError::context("invalid deployment journal", error))?;
    journal.validate()?;
    Ok(journal)
}

pub struct DeploymentLock {
    file: File,
}

impl DeploymentLock {
    pub fn acquire(
        path: &Path,
        trusted_root: &Path,
        expected_uid: u32,
        expected_gid: u32,
    ) -> Result<Self> {
        require_absolute_safe(path, "deployment lock path")?;
        validate_ancestry(path, trusted_root, expected_uid)?;
        let parent = path
            .parent()
            .ok_or_else(|| OpsError::message("deployment lock has no parent"))?;
        require_directory(
            parent,
            "deployment lock directory",
            Some(expected_uid),
            None,
            None,
        )?;
        let mut options = OpenOptions::new();
        options
            .read(true)
            .write(true)
            .create(true)
            .mode(0o600)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
        let file = options
            .open(path)
            .map_err(|error| OpsError::context("cannot open deployment lock", error))?;
        rustix::fs::fchmod(&file, rustix::fs::Mode::from_raw_mode(0o600))
            .map_err(|error| OpsError::context("cannot set deployment lock mode", error))?;
        rustix::fs::fchown(
            &file,
            Some(rustix::process::Uid::from_raw(expected_uid)),
            Some(rustix::process::Gid::from_raw(expected_gid)),
        )
        .map_err(|error| OpsError::context("cannot set deployment lock ownership", error))?;
        let metadata = file
            .metadata()
            .map_err(|error| OpsError::context("cannot inspect deployment lock", error))?;
        if !metadata.file_type().is_file()
            || metadata.uid() != expected_uid
            || metadata.gid() != expected_gid
            || metadata.permissions().mode() & 0o7777 != 0o600
        {
            return Err(OpsError::message("deployment lock identity differs"));
        }
        FileExt::try_lock_exclusive(&file)
            .map_err(|_| OpsError::message("another operation holds the global deployment lock"))?;
        sync_directory(parent)?;
        Ok(Self { file })
    }
}

impl Drop for DeploymentLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

fn allowed_transition(current: OperationPhase, next: OperationPhase) -> bool {
    use OperationPhase::*;
    matches!(
        (current, next),
        (Preflight, NoOp)
            | (Preflight, BackupVerified)
            | (NoOp, Accepted)
            | (BackupVerified, Fenced)
            | (Fenced, ServiceStopped)
            | (ServiceStopped, ArtifactsStaged)
            | (ArtifactsStaged, MigrationClassified)
            | (MigrationClassified, Activated)
            | (Activated, ServiceStarting)
            | (ServiceStarting, PostStartVerifying)
            | (PostStartVerifying, Accepted)
            | (Preflight, Abandoned)
            | (BackupVerified, Abandoned)
            | (Fenced, RolledBack)
            | (ServiceStopped, RolledBack)
            | (ArtifactsStaged, RolledBack)
            | (MigrationClassified, RolledBack)
            | (Activated, RolledBack)
            | (ServiceStarting, RolledBack)
            | (PostStartVerifying, RolledBack)
            | (MigrationClassified, RestoreRequired)
            | (Activated, RestoreRequired)
            | (ServiceStarting, RestoreRequired)
            | (PostStartVerifying, RestoreRequired)
            | (_, RecoveryBlocked)
    )
}

fn validate_migrations(values: &[MigrationIdentity]) -> Result<()> {
    let mut prior = None;
    for value in values {
        if prior.is_some_and(|version| value.version <= version)
            || value.name.is_empty()
            || value.name.len() > 256
        {
            return Err(OpsError::message(
                "migration identity list is not strictly ordered and bounded",
            ));
        }
        require_hex(&value.checksum, 64, "migration checksum")?;
        prior = Some(value.version);
    }
    Ok(())
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

fn bounded_failure(value: String) -> String {
    value
        .replace(['\r', '\n'], " ")
        .chars()
        .take(4096)
        .collect()
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
            let path = std::env::temp_dir().join(format!("lkjmc-ops-journal-{}", Uuid::new_v4()));
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
    fn global_lock_rejects_conflict_and_releases() -> Result<()> {
        let root = TestRoot::new()?;
        let path = root.0.join("deploy.lock");
        let first = DeploymentLock::acquire(&path, &root.0, effective_uid(), effective_gid())?;
        let error = DeploymentLock::acquire(&path, &root.0, effective_uid(), effective_gid())
            .err()
            .ok_or_else(|| OpsError::message("concurrent lock unexpectedly passed"))?;
        assert!(error.to_string().contains("global deployment lock"));
        drop(first);
        let _second = DeploymentLock::acquire(&path, &root.0, effective_uid(), effective_gid())?;
        Ok(())
    }

    #[test]
    fn recovery_classification_never_invents_binary_rollback() {
        let before = vec![migration(53)];
        assert_eq!(
            classify_recovery(&before, Some(&before)),
            RecoveryDecision::SafeBinaryRollback
        );
        assert_eq!(
            classify_recovery(&before, Some(&[migration(53), migration(54)])),
            RecoveryDecision::DataAwareRestoreRequired
        );
        assert_eq!(
            classify_recovery(&before, None),
            RecoveryDecision::RecoveryBlocked
        );
    }

    #[test]
    fn journal_preserves_first_failure_and_rejects_invalid_transition() -> Result<()> {
        let mut journal = fixture_journal();
        journal.transition(OperationPhase::BackupVerified)?;
        journal.record_failure("first causal failure\nwith detail");
        journal.record_failure("cleanup failure");
        assert_eq!(
            journal.first_failure.as_deref(),
            Some("first causal failure with detail")
        );
        assert!(journal.transition(OperationPhase::Accepted).is_err());
        Ok(())
    }

    fn fixture_journal() -> DeploymentJournal {
        DeploymentJournal {
            schema_version: 1,
            operation_id: Uuid::new_v4(),
            source_commit: "a".repeat(40),
            source_manifest_sha256: "b".repeat(64),
            target_commit: "b".repeat(40),
            manifest_sha256: "c".repeat(64),
            state_directory: PathBuf::from(format!(
                "/var/lib/private/lkjmc-deployments/{}",
                Uuid::new_v4()
            )),
            prior_release_root: PathBuf::from(format!("/opt/lkjmc/releases/{}", "a".repeat(40))),
            prior_unit_sha256: "d".repeat(64),
            prior_fence_dropin_sha256: "a".repeat(64),
            prior_plugins: BTreeMap::new(),
            migration_before: vec![migration(53)],
            migration_after: None,
            backup_path: PathBuf::from("/var/backups/lkjmc/pre-update"),
            backup: None,
            rollback_snapshot: Some("pre-update".to_string()),
            phase: OperationPhase::Preflight,
            first_failure: None,
            recovery_decision: None,
        }
    }

    fn migration(version: u32) -> MigrationIdentity {
        MigrationIdentity {
            version,
            name: format!("migration-{version}"),
            checksum: format!("{:064x}", version),
        }
    }
}
