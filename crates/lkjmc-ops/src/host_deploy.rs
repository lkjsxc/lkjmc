use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use lkjmc_core::config::LkjmcConfig;
use uuid::Uuid;

use crate::bootstrap;
use crate::database;
use crate::deploy::{
    execute_changed_update, recover_interrupted_update, ChangedUpdateEffects, DeployReceipt,
};
use crate::error::{OpsError, Result};
use crate::eula;
use crate::fence::{self, DeploymentFence, FenceOperation};
use crate::fleet::{service_identity, FleetSnapshot};
use crate::install::{
    self, validate_system_release_source, verify_installed_anchored,
    verify_installed_source_anchored, InstallFault, InstallScope, InstalledRelease,
};
use crate::journal::{
    read_journal, write_journal, BackupClosure, DeploymentJournal, DeploymentLock,
    MigrationIdentity, OperationPhase,
};
use crate::manifest::{sha256_file, VerifiedRelease};
use crate::process::{require_success, run_bounded, CommandSpec};
use crate::secure_fs::{
    atomic_symlink, atomic_write, copy_regular, create_directory, require_absolute_safe,
    require_directory, require_regular, sync_directory, validate_ancestry, MAX_CONTROL_FILE_BYTES,
};

pub(crate) const SYSTEMCTL: &str = "/usr/bin/systemctl";
pub(crate) const SERVICE: &str = "lkjmc-daemon.service";
const MAX_ARTIFACT_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_SECRET_BYTES: u64 = 4096;

#[derive(Debug, Clone)]
pub struct HostUpdateRequest {
    pub operation_id: Uuid,
    pub release_root: PathBuf,
    pub manifest_sha256: String,
    pub source_commit: String,
    pub source_manifest_sha256: String,
    pub config_path: PathBuf,
    pub backup_path: PathBuf,
    pub rollback_snapshot: String,
}

#[derive(Debug, Clone)]
pub struct HostRecoverRequest {
    pub operation_id: Uuid,
    pub release_root: PathBuf,
    pub manifest_sha256: String,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct HostLayout {
    pub(crate) releases: PathBuf,
    pub(crate) current: PathBuf,
    pub(crate) unit: PathBuf,
    pub(crate) fence_dropin: PathBuf,
    pub(crate) fence: PathBuf,
    pub(crate) permit: PathBuf,
    pub(crate) lock: PathBuf,
    pub(crate) state_root: PathBuf,
    pub(crate) policy: PathBuf,
}

impl HostLayout {
    pub(crate) fn from_config(config: &LkjmcConfig) -> Result<Self> {
        let install_root = PathBuf::from(&config.install_root);
        let config_root = PathBuf::from(&config.config_root);
        require_absolute_safe(&install_root, "configured install root")?;
        require_absolute_safe(&config_root, "configured configuration root")?;
        Ok(Self {
            releases: install_root.join("releases"),
            current: install_root.join("releases/current"),
            unit: PathBuf::from("/etc/systemd/system/lkjmc-daemon.service"),
            fence_dropin: PathBuf::from(
                "/etc/systemd/system/lkjmc-daemon.service.d/10-deployment-fence.conf",
            ),
            fence: config_root.join("deployment-fence.json"),
            permit: PathBuf::from("/run/lkjmc-deploy-start-permit.json"),
            lock: PathBuf::from("/run/lkjmc-deploy.lock"),
            state_root: PathBuf::from("/var/lib/private/lkjmc-deployments"),
            policy: config_root.join("minecraft-eula.accepted"),
        })
    }

    fn state_directory(&self, operation_id: Uuid) -> PathBuf {
        self.state_root.join(operation_id.to_string())
    }

    pub(crate) fn installation_state_root(&self) -> PathBuf {
        PathBuf::from("/var/lib/private/lkjmc-installations")
    }

    pub(crate) fn installation_state_directory(&self, operation_id: Uuid) -> PathBuf {
        self.installation_state_root()
            .join(operation_id.to_string())
    }
}

#[derive(Debug)]
struct Inspection {
    release: VerifiedRelease,
    source: InstalledRelease,
    config: LkjmcConfig,
    fleet: FleetSnapshot,
    layout: HostLayout,
    migration_before: Vec<MigrationIdentity>,
    prior_unit_sha256: String,
    prior_fence_dropin_sha256: String,
    prior_plugins: BTreeMap<String, String>,
    service_uid: u32,
    service_gid: u32,
    changed: bool,
}

pub fn update(request: HostUpdateRequest) -> Result<DeployReceipt> {
    crate::require_root()?;
    validate_update_request(&request)?;
    let retry_config = crate::fleet::read_config(&request.config_path)?;
    let retry_layout = HostLayout::from_config(&retry_config)?;
    if fs::symlink_metadata(retry_layout.state_directory(request.operation_id)).is_ok() {
        return recover(HostRecoverRequest {
            operation_id: request.operation_id,
            release_root: request.release_root,
            manifest_sha256: request.manifest_sha256,
            config_path: request.config_path,
        });
    }
    let first = inspect_update(&request, false)?;
    if !first.changed {
        verify_running_ops(&first.release)?;
        verify_exact_target(&first, &request.config_path)?;
        return DeployReceipt::no_op(&new_journal(&request, &first));
    }

    let _lock = DeploymentLock::acquire(&first.layout.lock, Path::new("/run"), 0, 0)?;
    let inspection = inspect_update(&request, true)?;
    if !inspection.changed {
        return Err(OpsError::message(
            "installed release changed while the deployment lock was acquired; retry preflight",
        ));
    }
    verify_running_ops(&inspection.release)?;
    ensure_no_unresolved_operation(&inspection.layout, request.operation_id)?;
    let journal = new_journal(&request, &inspection);
    let mut effects = HostEffects::new(request, inspection);
    execute_changed_update(journal, &mut effects)
}

pub fn recover(request: HostRecoverRequest) -> Result<DeployReceipt> {
    crate::require_root()?;
    validate_recover_request(&request)?;
    let config = crate::fleet::read_config(&request.config_path)?;
    let fleet = FleetSnapshot::from_config(&config)?;
    let service = service_identity(&config)?;
    let layout = HostLayout::from_config(&config)?;
    let _lock = DeploymentLock::acquire(&layout.lock, Path::new("/run"), 0, 0)?;
    let state_directory = layout.state_directory(request.operation_id);
    let journal_path = state_directory.join("journal.json");
    let mut journal = read_journal(&journal_path, 0, 0)?;
    if journal.operation_id != request.operation_id {
        return Err(OpsError::message(
            "recovery operation ID differs from the durable journal",
        ));
    }
    if journal.state_directory != state_directory {
        return Err(OpsError::message(
            "recovery journal state directory differs from the requested operation",
        ));
    }
    let fence_exists = path_exists(&layout.fence, "deployment fence")?;
    let permit_exists = path_exists(&layout.permit, "deployment start permit")?;
    if permit_exists && !fence_exists {
        return Err(OpsError::message(
            "an orphan deployment start permit blocks recovery",
        ));
    }
    if fence_exists
        && matches!(
            journal.phase,
            OperationPhase::Preflight | OperationPhase::Abandoned
        )
    {
        return Err(OpsError::message(
            "pre-fence deployment journal unexpectedly has a durable fence",
        ));
    }
    if fence_exists {
        let expected_fence = if journal.phase == OperationPhase::RolledBack {
            rollback_fence_from_journal(&journal)?
        } else {
            fence_from_journal(&journal)?
        };
        if fence::read_fence(&layout.fence, 0, 0)? != expected_fence {
            return Err(OpsError::message(
                "durable deployment fence differs from the recovery journal",
            ));
        }
    } else if !matches!(
        journal.phase,
        OperationPhase::Preflight
            | OperationPhase::BackupVerified
            | OperationPhase::Accepted
            | OperationPhase::Abandoned
            | OperationPhase::RolledBack
    ) {
        return Err(OpsError::message(
            "unresolved recovery journal has no matching durable fence",
        ));
    }
    let release = VerifiedRelease::load_anchored(&request.release_root, &request.manifest_sha256)?;
    validate_system_release_source(&release)?;
    if release.manifest.commit != journal.target_commit
        || release.manifest_sha256 != journal.manifest_sha256
    {
        return Err(OpsError::message(
            "recovery executable release differs from the durable target release",
        ));
    }
    verify_running_ops(&release)?;
    let source = load_source_release(
        &layout,
        &journal.source_commit,
        &journal.source_manifest_sha256,
        service.gid,
    )?;
    let mut database_connection = database::connect(&config, None)?;
    let persisted = database::persisted_inventory(&mut database_connection.client)?;
    fleet.compare_persisted(&persisted)?;
    let backup_exists = path_exists(&journal.backup_path, "planned deployment backup")?;
    if journal.backup.is_some() || backup_exists {
        let verified_backup = database::verify_backup(
            &config,
            &journal.backup_path,
            Some(&journal.source_commit),
            604_800,
        )?;
        if let Some(durable_backup) = &journal.backup {
            if &verified_backup != durable_backup {
                return Err(OpsError::message(
                    "verified backup closure differs from the recovery journal",
                ));
            }
        } else if journal.phase == OperationPhase::Preflight {
            journal.backup = Some(verified_backup);
        } else {
            return Err(OpsError::message(
                "a backup exists without a matching durable journal closure",
            ));
        }
    }
    let inspection = Inspection {
        release,
        source,
        config,
        fleet,
        layout,
        migration_before: journal.migration_before.clone(),
        prior_unit_sha256: journal.prior_unit_sha256.clone(),
        prior_fence_dropin_sha256: journal.prior_fence_dropin_sha256.clone(),
        prior_plugins: journal.prior_plugins.clone(),
        service_uid: service.uid,
        service_gid: service.gid,
        changed: true,
    };
    let mut effects = HostEffects::resume(
        request.config_path,
        service.uid,
        service.gid,
        inspection,
        state_directory,
    )?;
    if journal.phase == OperationPhase::Abandoned {
        effects.verify_source()?;
        return DeployReceipt::abandoned(&journal);
    }
    if !fence_exists
        && matches!(
            journal.phase,
            OperationPhase::Preflight | OperationPhase::BackupVerified
        )
    {
        if journal.backup.is_none() {
            let _ = database::cleanup_interrupted_backup_stages(&journal.backup_path)?;
        }
        effects.verify_source()?;
        if journal.first_failure.is_none() {
            journal.record_failure("deployment was interrupted before its durable fence");
        }
        journal.transition(OperationPhase::Abandoned)?;
        effects.persist_journal(&journal)?;
        return DeployReceipt::abandoned(&journal);
    }
    recover_interrupted_update(journal, &mut effects)
}

fn validate_update_request(request: &HostUpdateRequest) -> Result<()> {
    require_hex(&request.source_commit, 40, "source commit")?;
    require_hex(
        &request.source_manifest_sha256,
        64,
        "source manifest SHA-256",
    )?;
    require_hex(&request.manifest_sha256, 64, "target manifest SHA-256")?;
    require_absolute_safe(&request.release_root, "target release root")?;
    require_absolute_safe(&request.config_path, "configuration path")?;
    require_absolute_safe(&request.backup_path, "backup path")?;
    if !safe_label(&request.rollback_snapshot) {
        return Err(OpsError::message("rollback snapshot label is unsafe"));
    }
    Ok(())
}

fn validate_recover_request(request: &HostRecoverRequest) -> Result<()> {
    require_hex(&request.manifest_sha256, 64, "target manifest SHA-256")?;
    require_absolute_safe(&request.release_root, "target release root")?;
    require_absolute_safe(&request.config_path, "configuration path")?;
    Ok(())
}

fn inspect_update(request: &HostUpdateRequest, locked: bool) -> Result<Inspection> {
    let release = VerifiedRelease::load_anchored(&request.release_root, &request.manifest_sha256)?;
    validate_system_release_source(&release)?;
    if release.manifest.commit == request.source_commit
        && release.manifest_sha256 != request.source_manifest_sha256
    {
        return Err(OpsError::message(
            "same-commit update carries a different manifest identity",
        ));
    }
    let config = crate::fleet::read_config(&request.config_path)?;
    let fleet = FleetSnapshot::from_config(&config)?;
    let service = service_identity(&config)?;
    let layout = HostLayout::from_config(&config)?;
    validate_host_roots(&layout, &request.release_root)?;
    if request
        .backup_path
        .strip_prefix(Path::new("/var/backups/lkjmc"))
        .is_err()
    {
        return Err(OpsError::message(
            "changed-update backup must be under /var/backups/lkjmc",
        ));
    }
    let source = load_current_release(
        &layout,
        &request.source_commit,
        &request.source_manifest_sha256,
        service.gid,
    )?;
    let changed = release.manifest.commit != request.source_commit;
    if changed {
        validate_changed_destinations(request, &layout)?;
    }
    validate_configuration_effects(
        &config,
        &fleet,
        service.uid,
        service.gid,
        release.manifest.commit == request.source_commit,
        &layout.policy,
    )?;
    let mut connection = database::connect(&config, None)?;
    let persisted = database::persisted_inventory(&mut connection.client)?;
    fleet.compare_persisted(&persisted)?;
    let migration_before = database::migration_marker(&mut connection.client)?;
    drop(connection);
    let prior_unit_sha256 = verify_deployed_file(
        &layout.unit,
        &source.root.join("share/lkjmc-daemon.service"),
        0,
        0,
        0o644,
        "current systemd unit",
    )?;
    let prior_fence_dropin_sha256 = if changed {
        let _ = require_regular(
            &layout.fence_dropin,
            "current deployment fence drop-in",
            Some(0),
            Some(0),
            Some(0o644),
            MAX_CONTROL_FILE_BYTES,
        )?;
        sha256_file(&layout.fence_dropin)?
    } else {
        verify_deployed_file(
            &layout.fence_dropin,
            &source.root.join("share/lkjmc-deployment-fence.conf"),
            0,
            0,
            0o644,
            "current deployment fence drop-in",
        )?
    };
    let prior_plugins = verify_plugins(&fleet, &source.root, service.gid)?;
    verify_service_ready(&source, &request.config_path)?;
    if !changed {
        let _ = verify_installed_anchored(
            &source.root,
            &request.source_commit,
            &request.source_manifest_sha256,
            0,
            service.gid,
            0o750,
        )?;
    } else if !locked && fs::symlink_metadata(&layout.fence).is_ok() {
        return Err(OpsError::message(
            "an unresolved deployment fence blocks a changed update",
        ));
    }
    Ok(Inspection {
        release,
        source,
        config,
        fleet,
        layout,
        migration_before,
        prior_unit_sha256,
        prior_fence_dropin_sha256,
        prior_plugins,
        service_uid: service.uid,
        service_gid: service.gid,
        changed,
    })
}

fn validate_changed_destinations(request: &HostUpdateRequest, layout: &HostLayout) -> Result<()> {
    let backup_parent = request
        .backup_path
        .parent()
        .ok_or_else(|| OpsError::message("backup path has no parent"))?;
    require_directory(
        backup_parent,
        "backup destination parent",
        Some(0),
        Some(0),
        Some(0o700),
    )?;
    if fs::symlink_metadata(&request.backup_path).is_ok() {
        return Err(OpsError::message(
            "changed-update backup destination already exists",
        ));
    }
    let state_parent = layout
        .state_root
        .parent()
        .ok_or_else(|| OpsError::message("deployment state root has no parent"))?;
    require_directory(
        state_parent,
        "private deployment state parent",
        Some(0),
        Some(0),
        Some(0o700),
    )?;
    if fs::symlink_metadata(&layout.state_root).is_ok() {
        require_directory(
            &layout.state_root,
            "deployment state root",
            Some(0),
            Some(0),
            Some(0o700),
        )?;
    }
    Ok(())
}

fn validate_host_roots(layout: &HostLayout, release_root: &Path) -> Result<()> {
    let roots: [(&Path, &str); 4] = [
        (layout.releases.as_path(), "installed release root"),
        (
            layout
                .unit
                .parent()
                .ok_or_else(|| OpsError::message("systemd unit has no parent"))?,
            "systemd unit root",
        ),
        (
            layout
                .fence
                .parent()
                .ok_or_else(|| OpsError::message("fence has no parent"))?,
            "configuration root",
        ),
        (
            layout
                .permit
                .parent()
                .ok_or_else(|| OpsError::message("permit has no parent"))?,
            "runtime root",
        ),
    ];
    for (path, label) in roots {
        require_directory(path, label, Some(0), None, None)?;
        validate_ancestry(path, Path::new("/"), 0)?;
    }
    validate_ancestry(release_root, Path::new("/"), 0)?;
    Ok(())
}

fn load_current_release(
    layout: &HostLayout,
    expected_commit: &str,
    expected_manifest_sha256: &str,
    service_gid: u32,
) -> Result<InstalledRelease> {
    let metadata = fs::symlink_metadata(&layout.current)
        .map_err(|error| OpsError::context("cannot inspect current release pointer", error))?;
    if !metadata.file_type().is_symlink() || metadata.uid() != 0 {
        return Err(OpsError::message(
            "current release pointer is not a root-owned symlink",
        ));
    }
    let target = fs::read_link(&layout.current)
        .map_err(|error| OpsError::context("cannot read current release pointer", error))?;
    if target != Path::new(expected_commit) {
        return Err(OpsError::message(
            "current release pointer differs from the declared source commit",
        ));
    }
    load_source_release(
        layout,
        expected_commit,
        expected_manifest_sha256,
        service_gid,
    )
}

fn load_source_release(
    layout: &HostLayout,
    expected_commit: &str,
    expected_manifest_sha256: &str,
    service_gid: u32,
) -> Result<InstalledRelease> {
    let root = layout.releases.join(expected_commit);
    verify_installed_source_anchored(
        &root,
        expected_commit,
        expected_manifest_sha256,
        0,
        service_gid,
        0o750,
    )
}

pub(crate) fn verify_running_ops(release: &VerifiedRelease) -> Result<()> {
    let executable = fs::canonicalize("/proc/self/exe").map_err(|error| {
        OpsError::context("cannot identify running lkjmc-ops executable", error)
    })?;
    let metadata = fs::symlink_metadata(&executable)
        .map_err(|error| OpsError::context("cannot inspect running lkjmc-ops executable", error))?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.uid() != 0
        || metadata.mode() & 0o022 != 0
    {
        return Err(OpsError::message(
            "running lkjmc-ops executable ownership or mode is unsafe",
        ));
    }
    if sha256_file(&executable)? != release.artifact("lkjmc-ops")?.sha256 {
        return Err(OpsError::message(
            "running lkjmc-ops executable differs from the anchored target release",
        ));
    }
    Ok(())
}

pub(crate) fn validate_configuration_effects(
    config: &LkjmcConfig,
    fleet: &FleetSnapshot,
    service_uid: u32,
    service_gid: u32,
    require_materialized_eula: bool,
    policy_path: &Path,
) -> Result<()> {
    if eula::canonical_policy_path(config, service_gid)? != policy_path {
        return Err(OpsError::message(
            "deployment EULA policy path differs from the canonical configuration root",
        ));
    }
    if config.runtime.adapter != lkjmc_core::config::RuntimeAdapter::LocalProcess {
        return Err(OpsError::message(
            "packaged systemd deployment supports only the local-process runtime",
        ));
    }
    if !config.plugins.lkjmc.enabled {
        return Err(OpsError::message(
            "configured lkjmc Java integration is disabled",
        ));
    }
    let mut secret_paths = BTreeSet::from([
        config.database.secret_file.as_str(),
        config.network.forwarding.secret_file.as_str(),
    ]);
    if config.daemon_http.enabled {
        secret_paths.insert(config.daemon_http.token_file.as_str());
    }
    for path_text in secret_paths {
        let path = Path::new(path_text);
        require_absolute_safe(path, "deployment secret path")?;
        let metadata = require_regular(
            path,
            "deployment secret",
            Some(service_uid),
            Some(service_gid),
            Some(0o600),
            MAX_SECRET_BYTES,
        )?;
        if metadata.len() == 0 {
            return Err(OpsError::message(format!(
                "deployment secret is empty: {}",
                path.display()
            )));
        }
    }

    let referenced = fleet
        .instances()
        .flat_map(|instance| instance.asset_ids.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let declared = config
        .network
        .assets
        .iter()
        .map(|asset| asset.id.as_str())
        .collect::<BTreeSet<_>>();
    if referenced != declared {
        return Err(OpsError::message(
            "configured instance asset references differ from the immutable asset inventory",
        ));
    }
    for asset in &config.network.assets {
        if !asset.required {
            return Err(OpsError::message(format!(
                "optional runtime asset has no packaged update contract: {}",
                asset.id
            )));
        }
        let path = Path::new(&asset.path);
        require_absolute_safe(path, "immutable asset path")?;
        let metadata = require_regular(
            path,
            "immutable runtime asset",
            None,
            None,
            None,
            MAX_ARTIFACT_BYTES,
        )?;
        let owner = metadata.uid();
        if metadata.len() == 0
            || metadata.mode() & 0o022 != 0
            || (owner != 0 && owner != service_uid)
            || sha256_file(path)? != asset.sha256
        {
            return Err(OpsError::message(format!(
                "immutable runtime asset differs: {}",
                asset.id
            )));
        }
    }

    let credential_root = fleet.data_root.join("private/plugin-credentials");
    require_directory(
        &credential_root,
        "plugin credential root",
        Some(service_uid),
        Some(service_gid),
        Some(0o700),
    )?;
    let expected_credentials = fleet
        .credential_targets()
        .into_iter()
        .map(|target| target.path)
        .collect::<BTreeSet<_>>();
    let observed_credentials = fs::read_dir(&credential_root)
        .map_err(|error| OpsError::context("cannot enumerate plugin credential root", error))?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|error| OpsError::context("cannot read plugin credential entry", error))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    if expected_credentials != observed_credentials {
        return Err(OpsError::message(
            "plugin credential file set differs from the typed fleet",
        ));
    }
    for credential in expected_credentials {
        let metadata = require_regular(
            &credential,
            "plugin credential",
            Some(service_uid),
            Some(service_gid),
            Some(0o600),
            MAX_SECRET_BYTES,
        )?;
        if metadata.len() == 0 {
            return Err(OpsError::message(format!(
                "plugin credential is empty: {}",
                credential.display()
            )));
        }
    }

    if require_materialized_eula {
        eula::verify_materialized(fleet, policy_path, 0, service_uid, service_gid)?;
    } else {
        eula::verify_policy(policy_path, 0, service_gid)?;
    }
    Ok(())
}

fn verify_deployed_file(
    deployed: &Path,
    release_member: &Path,
    uid: u32,
    gid: u32,
    mode: u32,
    label: &str,
) -> Result<String> {
    let deployed_metadata = require_regular(
        deployed,
        label,
        Some(uid),
        Some(gid),
        Some(mode),
        MAX_CONTROL_FILE_BYTES,
    )?;
    let release_metadata = require_regular(
        release_member,
        "installed release member",
        Some(0),
        None,
        Some(0o640),
        MAX_CONTROL_FILE_BYTES,
    )?;
    let deployed_digest = sha256_file(deployed)?;
    if deployed_metadata.len() != release_metadata.len()
        || deployed_digest != sha256_file(release_member)?
    {
        return Err(OpsError::message(format!(
            "{label} differs from the current installed release"
        )));
    }
    Ok(deployed_digest)
}

fn verify_plugins(
    fleet: &FleetSnapshot,
    release_root: &Path,
    service_gid: u32,
) -> Result<BTreeMap<String, String>> {
    let mut observed = BTreeMap::new();
    for target in fleet.plugin_targets() {
        let parent = target
            .destination
            .parent()
            .ok_or_else(|| OpsError::message("plugin destination has no parent"))?;
        require_directory(
            parent,
            "managed plugin directory",
            None,
            Some(service_gid),
            Some(0o750),
        )?;
        let metadata = require_regular(
            &target.destination,
            "installed lkjmc plugin",
            Some(0),
            Some(service_gid),
            Some(0o640),
            MAX_ARTIFACT_BYTES,
        )?;
        let source = release_root.join("jars").join(target.artifact);
        let source_metadata = require_regular(
            &source,
            "release plugin",
            Some(0),
            Some(service_gid),
            Some(0o640),
            MAX_ARTIFACT_BYTES,
        )?;
        let digest = sha256_file(&target.destination)?;
        if metadata.len() != source_metadata.len() || digest != sha256_file(&source)? {
            return Err(OpsError::message(format!(
                "installed plugin differs for instance {}",
                target.instance_id.as_str()
            )));
        }
        observed.insert(target.instance_id.as_str().to_string(), digest);
    }
    Ok(observed)
}

fn verify_service_ready(release: &InstalledRelease, config_path: &Path) -> Result<()> {
    let state = systemd_state()?;
    if state.active_state != "active" || state.sub_state != "running" || state.main_pid == 0 {
        return Err(OpsError::message(
            "lkjmc systemd service is not active and running",
        ));
    }
    validate_cgroup_name(&state.control_group)?;
    let cli = release.root.join("bin/lkjmc");
    bootstrap::after_start(config_path, &cli, &release.commit, Duration::from_secs(120))?;
    Ok(())
}

fn verify_exact_target(inspection: &Inspection, config_path: &Path) -> Result<()> {
    if fs::symlink_metadata(&inspection.layout.fence).is_ok()
        || fs::symlink_metadata(&inspection.layout.permit).is_ok()
    {
        return Err(OpsError::message(
            "exact target is fenced or has an orphan start permit",
        ));
    }
    let _ = verify_installed_anchored(
        &inspection.source.root,
        &inspection.release.manifest.commit,
        &inspection.release.manifest_sha256,
        0,
        inspection
            .source
            .root
            .metadata()
            .map_err(|error| OpsError::context("cannot inspect installed target", error))?
            .gid(),
        0o750,
    )?;
    bootstrap::after_start(
        config_path,
        &inspection.source.root.join("bin/lkjmc"),
        &inspection.release.manifest.commit,
        Duration::from_secs(120),
    )?;
    Ok(())
}

fn ensure_no_unresolved_operation(layout: &HostLayout, operation_id: Uuid) -> Result<()> {
    if fs::symlink_metadata(&layout.fence).is_ok() {
        return Err(OpsError::message(
            "an unresolved deployment fence blocks the changed update",
        ));
    }
    if fs::symlink_metadata(&layout.permit).is_ok() {
        return Err(OpsError::message(
            "an orphan deployment start permit blocks the changed update",
        ));
    }
    if fs::symlink_metadata(&layout.state_root).is_err() {
        return Ok(());
    }
    require_directory(
        &layout.state_root,
        "deployment state root",
        Some(0),
        Some(0),
        Some(0o700),
    )?;
    let entries = fs::read_dir(&layout.state_root)
        .map_err(|error| OpsError::context("cannot enumerate deployment state", error))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| OpsError::context("cannot read deployment state entry", error))?;
    if entries.len() > 256 {
        return Err(OpsError::message(
            "deployment state exceeds the 256-operation retention bound",
        ));
    }
    for entry in entries {
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| OpsError::message("deployment state name is not UTF-8"))?;
        let observed_id = Uuid::parse_str(&name)
            .map_err(|_| OpsError::message("deployment state name is not a UUID"))?;
        require_directory(
            &entry.path(),
            "deployment operation state",
            Some(0),
            Some(0),
            Some(0o700),
        )?;
        let journal = read_journal(&entry.path().join("journal.json"), 0, 0)?;
        if journal.operation_id != observed_id || journal.state_directory != entry.path() {
            return Err(OpsError::message(
                "deployment operation state identity differs",
            ));
        }
        if observed_id == operation_id {
            return Err(OpsError::message(format!(
                "deployment operation state already exists; use deploy recover with operation {operation_id}"
            )));
        }
        if !matches!(
            journal.phase,
            OperationPhase::Accepted | OperationPhase::Abandoned | OperationPhase::RolledBack
        ) {
            return Err(OpsError::message(format!(
                "unresolved deployment operation {observed_id} blocks a changed update"
            )));
        }
    }
    Ok(())
}

fn new_journal(request: &HostUpdateRequest, inspection: &Inspection) -> DeploymentJournal {
    DeploymentJournal {
        schema_version: 1,
        operation_id: request.operation_id,
        source_commit: request.source_commit.clone(),
        source_manifest_sha256: request.source_manifest_sha256.clone(),
        target_commit: inspection.release.manifest.commit.clone(),
        manifest_sha256: inspection.release.manifest_sha256.clone(),
        state_directory: inspection.layout.state_directory(request.operation_id),
        prior_release_root: inspection.source.root.clone(),
        prior_unit_sha256: inspection.prior_unit_sha256.clone(),
        prior_fence_dropin_sha256: inspection.prior_fence_dropin_sha256.clone(),
        prior_plugins: inspection.prior_plugins.clone(),
        migration_before: inspection.migration_before.clone(),
        migration_after: None,
        backup_path: request.backup_path.clone(),
        backup: None,
        rollback_snapshot: Some(request.rollback_snapshot.clone()),
        phase: OperationPhase::Preflight,
        first_failure: None,
        recovery_decision: None,
    }
}

struct HostEffects {
    config_path: PathBuf,
    backup_path: Option<PathBuf>,
    service_uid: u32,
    service_gid: u32,
    inspection: Inspection,
    state_directory: PathBuf,
    eula_receipt: Option<eula::EulaReceipt>,
    state_prepared: bool,
}

impl HostEffects {
    fn new(request: HostUpdateRequest, inspection: Inspection) -> Self {
        let state_directory = inspection.layout.state_directory(request.operation_id);
        let service_uid = inspection.service_uid;
        let service_gid = inspection.service_gid;
        Self {
            config_path: request.config_path,
            backup_path: Some(request.backup_path),
            service_uid,
            service_gid,
            inspection,
            state_directory,
            eula_receipt: None,
            state_prepared: false,
        }
    }

    fn resume(
        config_path: PathBuf,
        service_uid: u32,
        service_gid: u32,
        inspection: Inspection,
        state_directory: PathBuf,
    ) -> Result<Self> {
        require_directory(
            &state_directory,
            "deployment state directory",
            Some(0),
            Some(0),
            Some(0o700),
        )?;
        Ok(Self {
            config_path,
            backup_path: None,
            service_uid,
            service_gid,
            inspection,
            state_directory,
            eula_receipt: None,
            state_prepared: true,
        })
    }

    fn journal_path(&self) -> PathBuf {
        self.state_directory.join("journal.json")
    }

    fn prior_unit_path(&self) -> PathBuf {
        self.state_directory.join("prior-unit")
    }

    fn prior_dropin_path(&self) -> PathBuf {
        self.state_directory.join("prior-fence-dropin")
    }

    fn prior_plugin_path(&self, instance_id: &str) -> PathBuf {
        self.state_directory
            .join("prior-plugins")
            .join(format!("{instance_id}.jar"))
    }

    fn target_root(&self) -> PathBuf {
        self.inspection
            .layout
            .releases
            .join(&self.inspection.release.manifest.commit)
    }

    fn prepare_state(&mut self, journal: &DeploymentJournal) -> Result<()> {
        let state_parent = self
            .inspection
            .layout
            .state_root
            .parent()
            .ok_or_else(|| OpsError::message("deployment state root has no parent"))?;
        require_directory(
            state_parent,
            "private deployment state parent",
            Some(0),
            Some(0),
            Some(0o700),
        )?;
        if fs::symlink_metadata(&self.inspection.layout.state_root).is_err() {
            create_directory(&self.inspection.layout.state_root, 0o700, 0, 0)?;
            sync_directory(state_parent)?;
        } else {
            require_directory(
                &self.inspection.layout.state_root,
                "deployment state root",
                Some(0),
                Some(0),
                Some(0o700),
            )?;
        }
        create_directory(&self.state_directory, 0o700, 0, 0)?;
        sync_directory(&self.inspection.layout.state_root)?;
        write_journal(&self.journal_path(), journal, 0, 0)?;
        let preparation = (|| {
            let receipt = eula::materialize(
                &self.inspection.fleet,
                &self.inspection.layout.policy,
                0,
                self.service_uid,
                self.service_gid,
            )?;
            eula::verify_materialized(
                &self.inspection.fleet,
                &self.inspection.layout.policy,
                0,
                self.service_uid,
                self.service_gid,
            )?;
            self.eula_receipt = Some(receipt);
            self.save_prior_runtime_files()?;
            let receipt = self
                .eula_receipt
                .as_ref()
                .ok_or_else(|| OpsError::message("EULA receipt disappeared during preflight"))?;
            let mut raw = serde_json::to_vec(receipt)?;
            raw.push(b'\n');
            atomic_write(
                &self.state_directory.join("eula-receipt.json"),
                &raw,
                0o600,
                0,
                0,
            )?;
            sync_directory(&self.state_directory)
        })();
        if let Err(error) = preparation {
            let mut failed = journal.clone();
            failed.record_failure(error.to_string());
            let _ = write_journal(&self.journal_path(), &failed, 0, 0);
            return Err(error);
        }
        self.state_prepared = true;
        Ok(())
    }

    fn save_prior_runtime_files(&self) -> Result<()> {
        copy_regular(
            &self.inspection.layout.unit,
            &self.prior_unit_path(),
            0o600,
            0,
            0,
            MAX_CONTROL_FILE_BYTES,
        )?;
        copy_regular(
            &self.inspection.layout.fence_dropin,
            &self.prior_dropin_path(),
            0o600,
            0,
            0,
            MAX_CONTROL_FILE_BYTES,
        )?;
        let plugin_root = self.state_directory.join("prior-plugins");
        create_directory(&plugin_root, 0o700, 0, 0)?;
        for target in self.inspection.fleet.plugin_targets() {
            copy_regular(
                &target.destination,
                &self.prior_plugin_path(target.instance_id.as_str()),
                0o600,
                0,
                0,
                MAX_ARTIFACT_BYTES,
            )?;
        }
        if sha256_file(&self.prior_unit_path())? != self.inspection.prior_unit_sha256
            || sha256_file(&self.prior_dropin_path())? != self.inspection.prior_fence_dropin_sha256
        {
            return Err(OpsError::message(
                "saved prior systemd files differ from preflight identity",
            ));
        }
        for (id, digest) in &self.inspection.prior_plugins {
            if sha256_file(&self.prior_plugin_path(id))? != *digest {
                return Err(OpsError::message(format!(
                    "saved prior plugin differs for instance {id}"
                )));
            }
        }
        Ok(())
    }

    fn publish_runtime_files(&self, release_root: &Path) -> Result<()> {
        for target in self.inspection.fleet.plugin_targets() {
            let source = release_root.join("jars").join(target.artifact);
            copy_regular(
                &source,
                &target.destination,
                0o640,
                0,
                self.service_gid,
                MAX_ARTIFACT_BYTES,
            )?;
        }
        copy_regular(
            &release_root.join("share/lkjmc-daemon.service"),
            &self.inspection.layout.unit,
            0o644,
            0,
            0,
            MAX_CONTROL_FILE_BYTES,
        )?;
        copy_regular(
            &release_root.join("share/lkjmc-deployment-fence.conf"),
            &self.inspection.layout.fence_dropin,
            0o644,
            0,
            0,
            MAX_CONTROL_FILE_BYTES,
        )?;
        Ok(())
    }

    fn verify_runtime_files(&self, release_root: &Path) -> Result<()> {
        let _ = verify_deployed_file(
            &self.inspection.layout.unit,
            &release_root.join("share/lkjmc-daemon.service"),
            0,
            0,
            0o644,
            "deployed systemd unit",
        )?;
        let _ = verify_deployed_file(
            &self.inspection.layout.fence_dropin,
            &release_root.join("share/lkjmc-deployment-fence.conf"),
            0,
            0,
            0o644,
            "deployed fence drop-in",
        )?;
        let _ = verify_plugins(&self.inspection.fleet, release_root, self.service_gid)?;
        Ok(())
    }

    fn verify_database_inventory(&self) -> Result<Vec<MigrationIdentity>> {
        let mut connection = database::connect(&self.inspection.config, None)?;
        let persisted = database::persisted_inventory(&mut connection.client)?;
        self.inspection.fleet.compare_persisted(&persisted)?;
        database::migration_marker(&mut connection.client)
    }
}

impl ChangedUpdateEffects for HostEffects {
    fn create_verified_backup(&mut self) -> Result<BackupClosure> {
        let backup_path = self
            .backup_path
            .as_ref()
            .ok_or_else(|| OpsError::message("recovery cannot create a replacement backup"))?;
        let backup_parent = backup_path
            .parent()
            .ok_or_else(|| OpsError::message("backup path has no parent"))?;
        require_directory(
            backup_parent,
            "backup destination parent",
            Some(0),
            Some(0),
            Some(0o700),
        )?;
        let closure = database::create_backup(
            &self.inspection.config,
            backup_path,
            &self.inspection.source.commit,
        )?;
        Ok(closure)
    }

    fn persist_journal(&mut self, journal: &DeploymentJournal) -> Result<()> {
        if !self.state_prepared {
            self.prepare_state(journal)?;
        }
        write_journal(&self.journal_path(), journal, 0, 0)
    }

    fn write_fence(&mut self, journal: &DeploymentJournal) -> Result<()> {
        let value = fence_from_journal(journal)?;
        fence::write_fence(&self.inspection.layout.fence, &value, 0, 0)
    }

    fn stop_service(&mut self) -> Result<()> {
        stop_service()
    }

    fn stage_artifacts(&mut self) -> Result<()> {
        let result = install::install(
            &self.inspection.release,
            &self.target_root(),
            InstallScope::System {
                service_uid: self.service_uid,
                service_gid: self.service_gid,
            },
            InstallFault::None,
        )?;
        if !matches!(
            result,
            install::InstallResult::Updated | install::InstallResult::NoOp
        ) {
            return Err(OpsError::message("target artifact staging did not finish"));
        }
        let _ = verify_installed_anchored(
            &self.target_root(),
            &self.inspection.release.manifest.commit,
            &self.inspection.release.manifest_sha256,
            0,
            self.service_gid,
            0o750,
        )?;
        Ok(())
    }

    fn apply_migrations(&mut self) -> Result<Vec<MigrationIdentity>> {
        database::apply_migrations(&self.inspection.config)
    }

    fn activate_target(&mut self) -> Result<()> {
        self.publish_runtime_files(&self.target_root())?;
        atomic_symlink(
            Path::new(&self.inspection.release.manifest.commit),
            &self.inspection.layout.current,
            0,
        )?;
        systemctl(&["daemon-reload"], Duration::from_secs(60))?;
        self.verify_runtime_files(&self.target_root())
    }

    fn start_target_once(&mut self, journal: &DeploymentJournal) -> Result<()> {
        let active_fence = fence_from_journal(journal)?;
        fence::write_permit(&self.inspection.layout.permit, &active_fence, 0, 0)?;
        systemctl(&["start", SERVICE], Duration::from_secs(1500))?;
        require_service_running()
    }

    fn verify_target(&mut self) -> Result<()> {
        let target_root = self.target_root();
        verify_current_pointer(
            &self.inspection.layout.current,
            &self.inspection.release.manifest.commit,
        )?;
        let _ = verify_installed_anchored(
            &target_root,
            &self.inspection.release.manifest.commit,
            &self.inspection.release.manifest_sha256,
            0,
            self.service_gid,
            0o750,
        )?;
        self.verify_runtime_files(&target_root)?;
        eula::verify_materialized(
            &self.inspection.fleet,
            &self.inspection.layout.policy,
            0,
            self.service_uid,
            self.service_gid,
        )?;
        let _ = self.verify_database_inventory()?;
        require_service_running()?;
        bootstrap::after_start(
            &self.config_path,
            &target_root.join("bin/lkjmc"),
            &self.inspection.release.manifest.commit,
            Duration::from_secs(120),
        )?;
        Ok(())
    }

    fn observe_migrations(&mut self) -> Result<Vec<MigrationIdentity>> {
        let mut connection = database::connect(&self.inspection.config, None)?;
        database::migration_marker(&mut connection.client)
    }

    fn restore_source(&mut self, journal: &DeploymentJournal) -> Result<()> {
        stop_service()?;
        let target_fence = fence_from_journal(journal)?;
        let _ = fence::remove_matching_permit(&self.inspection.layout.permit, &target_fence, 0, 0)?;
        if sha256_file(&self.prior_unit_path())? != journal.prior_unit_sha256
            || sha256_file(&self.prior_dropin_path())? != journal.prior_fence_dropin_sha256
        {
            return Err(OpsError::message(
                "saved systemd rollback inputs differ from the journal",
            ));
        }
        for (id, digest) in &journal.prior_plugins {
            if sha256_file(&self.prior_plugin_path(id))? != *digest {
                return Err(OpsError::message(format!(
                    "saved plugin rollback input differs for instance {id}"
                )));
            }
        }
        copy_regular(
            &self.prior_unit_path(),
            &self.inspection.layout.unit,
            0o644,
            0,
            0,
            MAX_CONTROL_FILE_BYTES,
        )?;
        copy_regular(
            &self.prior_dropin_path(),
            &self.inspection.layout.fence_dropin,
            0o644,
            0,
            0,
            MAX_CONTROL_FILE_BYTES,
        )?;
        for target in self.inspection.fleet.plugin_targets() {
            copy_regular(
                &self.prior_plugin_path(target.instance_id.as_str()),
                &target.destination,
                0o640,
                0,
                self.service_gid,
                MAX_ARTIFACT_BYTES,
            )?;
        }
        atomic_symlink(
            Path::new(&journal.source_commit),
            &self.inspection.layout.current,
            0,
        )?;
        systemctl(&["daemon-reload"], Duration::from_secs(60))?;
        let rollback_fence = rollback_fence_from_journal(journal)?;
        fence::write_fence(&self.inspection.layout.fence, &rollback_fence, 0, 0)?;
        fence::write_permit(&self.inspection.layout.permit, &rollback_fence, 0, 0)?;
        systemctl(&["start", SERVICE], Duration::from_secs(1500))?;
        require_service_running()
    }

    fn verify_source(&mut self) -> Result<()> {
        verify_current_pointer(
            &self.inspection.layout.current,
            &self.inspection.source.commit,
        )?;
        let _ = verify_installed_source_anchored(
            &self.inspection.source.root,
            &self.inspection.source.commit,
            &self.inspection.source.manifest_sha256,
            0,
            self.service_gid,
            0o750,
        )?;
        if sha256_file(&self.inspection.layout.unit)? != self.inspection.prior_unit_sha256
            || sha256_file(&self.inspection.layout.fence_dropin)?
                != self.inspection.prior_fence_dropin_sha256
        {
            return Err(OpsError::message(
                "restored systemd files differ from the prior identity",
            ));
        }
        for target in self.inspection.fleet.plugin_targets() {
            let expected = self
                .inspection
                .prior_plugins
                .get(target.instance_id.as_str())
                .ok_or_else(|| OpsError::message("prior plugin identity disappeared"))?;
            if sha256_file(&target.destination)? != *expected {
                return Err(OpsError::message(format!(
                    "restored plugin differs for instance {}",
                    target.instance_id.as_str()
                )));
            }
        }
        let observed = self.verify_database_inventory()?;
        if observed != self.inspection.migration_before {
            return Err(OpsError::message(
                "PostgreSQL migration ledger changed during safe rollback",
            ));
        }
        require_service_running()?;
        bootstrap::after_start(
            &self.config_path,
            &self.inspection.source.root.join("bin/lkjmc"),
            &self.inspection.source.commit,
            Duration::from_secs(120),
        )?;
        Ok(())
    }

    fn clear_fence(&mut self) -> Result<()> {
        if fs::symlink_metadata(&self.inspection.layout.permit).is_ok() {
            return Err(OpsError::message(
                "deployment start permit was not consumed by systemd",
            ));
        }
        if fs::symlink_metadata(&self.inspection.layout.fence).is_err() {
            return Ok(());
        }
        fence::remove_fence(&self.inspection.layout.fence, 0, 0)
    }
}

#[derive(Debug)]
struct SystemdState {
    active_state: String,
    sub_state: String,
    control_group: String,
    main_pid: u32,
}

pub(crate) fn systemctl(arguments: &[&str], timeout: Duration) -> Result<()> {
    let output = require_success(
        run_bounded(&CommandSpec {
            executable: PathBuf::from(SYSTEMCTL),
            arguments: arguments.iter().map(|value| (*value).to_string()).collect(),
            environment: BTreeMap::new(),
            stdin: Vec::new(),
            timeout,
            max_output_bytes: 2 * 1024 * 1024,
        })?,
        "systemd operation",
    )?;
    if !output.stdout.is_empty() && output.stdout.len() > 2 * 1024 * 1024 {
        return Err(OpsError::message("systemd output exceeded its bound"));
    }
    Ok(())
}

fn systemd_state() -> Result<SystemdState> {
    let output = require_success(
        run_bounded(&CommandSpec {
            executable: PathBuf::from(SYSTEMCTL),
            arguments: vec![
                "show".to_string(),
                SERVICE.to_string(),
                "--property=ActiveState,SubState,ControlGroup,MainPID".to_string(),
            ],
            environment: BTreeMap::new(),
            stdin: Vec::new(),
            timeout: Duration::from_secs(30),
            max_output_bytes: 64 * 1024,
        })?,
        "systemd state observation",
    )?;
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|_| OpsError::message("systemd state output is not UTF-8"))?;
    let mut values = BTreeMap::new();
    for line in text.lines() {
        let (name, value) = line
            .split_once('=')
            .ok_or_else(|| OpsError::message("systemd state output is malformed"))?;
        if values.insert(name, value).is_some() {
            return Err(OpsError::message("systemd state output duplicates a field"));
        }
    }
    let exact = ["ActiveState", "ControlGroup", "MainPID", "SubState"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    if values.keys().copied().collect::<BTreeSet<_>>() != exact {
        return Err(OpsError::message("systemd state field set differs"));
    }
    let main_pid = values["MainPID"]
        .parse::<u32>()
        .map_err(|_| OpsError::message("systemd MainPID is invalid"))?;
    Ok(SystemdState {
        active_state: values["ActiveState"].to_string(),
        sub_state: values["SubState"].to_string(),
        control_group: values["ControlGroup"].to_string(),
        main_pid,
    })
}

pub(crate) fn stop_service() -> Result<()> {
    let before = systemd_state()?;
    if before.active_state != "inactive" {
        validate_cgroup_name(&before.control_group)?;
        systemctl(&["stop", SERVICE], Duration::from_secs(180))?;
    }
    let after = systemd_state()?;
    if after.active_state != "inactive" || after.main_pid != 0 {
        return Err(OpsError::message(
            "systemd did not reach inactive state with no MainPID",
        ));
    }
    if !before.control_group.is_empty() {
        verify_cgroup_empty(&before.control_group)?;
    }
    Ok(())
}

pub(crate) fn require_service_running() -> Result<()> {
    let _ = require_service_running_state()?;
    Ok(())
}

pub(crate) fn require_service_running_identity(
    expected_executable: &Path,
    expected_uid: u32,
) -> Result<()> {
    let state = require_service_running_state()?;
    let expected = fs::canonicalize(expected_executable).map_err(|error| {
        OpsError::context("cannot resolve expected lkjmc daemon executable", error)
    })?;
    let expected_metadata = fs::metadata(&expected).map_err(|error| {
        OpsError::context("cannot inspect expected lkjmc daemon executable", error)
    })?;
    if !expected_metadata.file_type().is_file()
        || expected_metadata.uid() != 0
        || expected_metadata.mode() & 0o022 != 0
        || expected_metadata.mode() & 0o111 == 0
    {
        return Err(OpsError::message(
            "expected lkjmc daemon executable identity or mode is unsafe",
        ));
    }
    let observed = observe_process_identity(state.main_pid)?;
    if observed.executable != expected
        || observed.executable_device != expected_metadata.dev()
        || observed.executable_inode != expected_metadata.ino()
    {
        return Err(OpsError::message(
            "systemd main process executable differs from the accepted lkjmc daemon release",
        ));
    }
    if observed.uid != expected_uid {
        return Err(OpsError::message(
            "systemd main process UID differs from the accepted lkjmc service identity",
        ));
    }
    let after_state = require_service_running_state()?;
    let after = observe_process_identity(after_state.main_pid)?;
    if after_state.main_pid != state.main_pid
        || after_state.control_group != state.control_group
        || after != observed
    {
        return Err(OpsError::message(
            "systemd main process identity changed during acceptance observation",
        ));
    }
    Ok(())
}

fn require_service_running_state() -> Result<SystemdState> {
    let state = systemd_state()?;
    if state.active_state != "active" || state.sub_state != "running" || state.main_pid == 0 {
        return Err(OpsError::message(
            "systemd service did not reach active/running with a MainPID",
        ));
    }
    validate_cgroup_name(&state.control_group)?;
    Ok(state)
}

fn process_status_uid(status: &str) -> Option<u32> {
    let value = status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:\t"))?;
    let uid = value.split_whitespace().next()?;
    uid.parse().ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessIdentity {
    executable: PathBuf,
    executable_device: u64,
    executable_inode: u64,
    uid: u32,
    start_ticks: u64,
}

fn observe_process_identity(pid: u32) -> Result<ProcessIdentity> {
    if pid == 0 {
        return Err(OpsError::message("systemd main process PID is zero"));
    }
    let process_root = PathBuf::from(format!("/proc/{pid}"));
    let executable = fs::canonicalize(process_root.join("exe")).map_err(|error| {
        OpsError::context("cannot resolve systemd main process executable", error)
    })?;
    let executable_metadata = fs::metadata(&executable).map_err(|error| {
        OpsError::context("cannot inspect systemd main process executable", error)
    })?;
    let status = read_proc_text(
        &process_root.join("status"),
        "systemd main process identity",
    )?;
    let stat = read_proc_text(
        &process_root.join("stat"),
        "systemd main process start identity",
    )?;
    Ok(ProcessIdentity {
        executable,
        executable_device: executable_metadata.dev(),
        executable_inode: executable_metadata.ino(),
        uid: process_status_uid(&status)
            .ok_or_else(|| OpsError::message("systemd main process UID is malformed"))?,
        start_ticks: process_start_ticks(&stat, pid)
            .ok_or_else(|| OpsError::message("systemd main process start identity is malformed"))?,
    })
}

fn read_proc_text(path: &Path, label: &str) -> Result<String> {
    let raw = fs::read(path)
        .map_err(|error| OpsError::context(&format!("cannot inspect {label}"), error))?;
    if raw.len() > 64 * 1024 {
        return Err(OpsError::message(format!("{label} exceeds its bound")));
    }
    String::from_utf8(raw).map_err(|_| OpsError::message(format!("{label} is not UTF-8")))
}

fn process_start_ticks(stat: &str, pid: u32) -> Option<u64> {
    let (recorded_pid, _) = stat.split_once(" (")?;
    if recorded_pid.parse::<u32>().ok()? != pid {
        return None;
    }
    let close = stat.rfind(')')?;
    let fields = stat
        .get(close + 1..)?
        .split_whitespace()
        .collect::<Vec<_>>();
    fields.get(19)?.parse().ok()
}

fn validate_cgroup_name(value: &str) -> Result<()> {
    let path = Path::new(value);
    if value.is_empty()
        || !path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
        || !value.ends_with(SERVICE)
    {
        return Err(OpsError::message(
            "systemd control group identity is unsafe or belongs to another unit",
        ));
    }
    Ok(())
}

fn verify_cgroup_empty(control_group: &str) -> Result<()> {
    validate_cgroup_name(control_group)?;
    let path = Path::new("/sys/fs/cgroup")
        .join(control_group.trim_start_matches('/'))
        .join("cgroup.procs");
    let raw = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(OpsError::context(
                "cannot inspect stopped service cgroup",
                error,
            ))
        }
    };
    if raw.lines().any(|line| !line.trim().is_empty()) {
        return Err(OpsError::message(
            "a process survived in the stopped lkjmc service cgroup",
        ));
    }
    Ok(())
}

fn fence_from_journal(journal: &DeploymentJournal) -> Result<DeploymentFence> {
    Ok(DeploymentFence {
        schema_version: 1,
        operation: FenceOperation::Update,
        operation_id: journal.operation_id,
        from_commit: Some(journal.source_commit.clone()),
        to_commit: journal.target_commit.clone(),
        manifest_sha256: journal.manifest_sha256.clone(),
        state_directory: journal.state_directory.clone(),
        backup: Some(backup_root(journal)?.to_path_buf()),
        rollback_snapshot: Some(
            journal
                .rollback_snapshot
                .clone()
                .ok_or_else(|| OpsError::message("journal has no rollback snapshot assertion"))?,
        ),
    })
}

fn rollback_fence_from_journal(journal: &DeploymentJournal) -> Result<DeploymentFence> {
    Ok(DeploymentFence {
        schema_version: 1,
        operation: FenceOperation::Update,
        operation_id: journal.operation_id,
        from_commit: Some(journal.target_commit.clone()),
        to_commit: journal.source_commit.clone(),
        manifest_sha256: journal.source_manifest_sha256.clone(),
        state_directory: journal.state_directory.clone(),
        backup: Some(backup_root(journal)?.to_path_buf()),
        rollback_snapshot: Some(
            journal
                .rollback_snapshot
                .clone()
                .ok_or_else(|| OpsError::message("journal has no rollback snapshot assertion"))?,
        ),
    })
}

fn backup_root(journal: &DeploymentJournal) -> Result<&Path> {
    journal
        .backup
        .as_ref()
        .and_then(|backup| backup.dump.parent())
        .ok_or_else(|| OpsError::message("journal has no verified backup root"))
}

fn verify_current_pointer(path: &Path, commit: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| OpsError::context("cannot inspect release pointer", error))?;
    if !metadata.file_type().is_symlink()
        || metadata.uid() != 0
        || fs::read_link(path)
            .map_err(|error| OpsError::context("cannot read release pointer", error))?
            != Path::new(commit)
    {
        return Err(OpsError::message("current release pointer differs"));
    }
    Ok(())
}

fn safe_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

fn path_exists(path: &Path, label: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(OpsError::context(&format!("cannot inspect {label}"), error)),
    }
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

#[cfg(test)]
mod tests {
    use super::{process_start_ticks, process_status_uid};

    #[test]
    fn process_status_uid_uses_the_real_uid_field() {
        let status =
            "Name:\tlkjmc-daemon\nUid:\t1234\t1234\t1234\t1234\nGid:\t5678\t5678\t5678\t5678\n";
        assert_eq!(process_status_uid(status), Some(1234));
        assert_eq!(process_status_uid("Name:\tmissing\n"), None);
        assert_eq!(process_status_uid("Uid:\tnot-a-number\n"), None);
    }

    #[test]
    fn process_start_ticks_binds_the_reported_pid_and_start_field() {
        let stat = "123 (lkjmc daemon) S 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 444 20";
        assert_eq!(process_start_ticks(stat, 123), Some(444));
        assert_eq!(process_start_ticks(stat, 124), None);
        assert_eq!(process_start_ticks("123 malformed", 123), None);
    }
}
