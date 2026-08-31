use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use lkjmc_core::config::{AssetKind, LkjmcConfig, RuntimeAdapter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{OpsError, Result};
use crate::fleet::FleetSnapshot;
use crate::host_deploy::{self, HostLayout};
use crate::journal::DeploymentLock;
use crate::manifest::{sha256_bytes, sha256_file, VerifiedRelease};
use crate::process::{run_bounded, CommandSpec};
use crate::secure_fs::{
    atomic_symlink, atomic_write, copy_regular, create_directory, read_regular,
    require_absolute_safe, require_directory, require_regular, sync_directory, validate_ancestry,
    MAX_CONTROL_FILE_BYTES,
};

const INPUT_MAX_BYTES: u64 = 1024 * 1024;
const MAX_RUNTIME_ASSET_BYTES: u64 = 1024 * 1024 * 1024;

const INSTALL_ROOT: &str = "/opt/lkjmc";
const CONFIG_ROOT: &str = "/etc/lkjmc";
const DATA_ROOT: &str = "/var/lib/lkjmc";
const LOG_ROOT: &str = "/var/log/lkjmc";
const BACKUP_ROOT: &str = "/var/backups/lkjmc";
const RUNTIME_ASSET_ROOT: &str = "/opt/lkjmc/runtime-assets";
const JAR_ROOT: &str = "/opt/lkjmc/jars";
const SOCKET_PATH: &str = "/run/lkjmc/daemon.sock";
const DATABASE_SECRET: &str = "/etc/lkjmc/database.secret";
const FORWARDING_SECRET: &str = "/etc/lkjmc/forwarding.secret";
const HTTP_TOKEN: &str = "/etc/lkjmc/daemon-http.token";
const SERVICE_USER: &str = "lkjmc";
const SERVICE_GROUP: &str = "lkjmc";
const SERVICE_HOME: &str = "/var/lib/lkjmc";
const SERVICE_SHELL: &str = "/usr/sbin/nologin";
const POSTGRES_ADMIN: &str = "postgres";
const POSTGRES_SOCKET: &str = "/var/run/postgresql";
const PLUGIN_HEARTBEAT_SCOPE: &str = "lkjmc.instance.heartbeat";
const PLUGIN_HEARTBEAT_EXPIRY_SECONDS: i64 = 365 * 24 * 60 * 60;

#[derive(Debug, Clone)]
pub struct HostInstallRequest {
    pub input_path: PathBuf,
    pub input_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostInstallInput {
    pub schema_version: u32,
    pub operation_id: Uuid,
    pub release: ReleaseInput,
    pub configuration: ImmutableFileInput,
    pub assets: Vec<RuntimeAssetInput>,
    pub roots: InstallRoots,
    pub service: ServiceContract,
    pub postgres: PostgresContract,
    pub capacity: CapacityContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseInput {
    pub root: PathBuf,
    pub commit: String,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImmutableFileInput {
    pub path: PathBuf,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeAssetInput {
    pub id: String,
    pub kind: AssetKind,
    pub version: String,
    pub source_identity: String,
    pub source: ImmutableFileInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallRoots {
    pub install_root: PathBuf,
    pub config_root: PathBuf,
    pub data_root: PathBuf,
    pub log_root: PathBuf,
    pub backup_root: PathBuf,
    pub runtime_asset_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceContract {
    pub user: String,
    pub group: String,
    pub uid: u32,
    pub gid: u32,
    pub home: PathBuf,
    pub shell: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostgresContract {
    pub administrative_user: String,
    pub socket_directory: PathBuf,
    pub role: String,
    pub database: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapacityContract {
    pub minimum_free_mib: u64,
    pub minimum_memory_mib: u64,
    pub minimum_processes: u32,
    pub minimum_open_files: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct InstallInspection {
    pub input: HostInstallInput,
    pub input_sha256: String,
    pub release: VerifiedRelease,
    pub config_bytes: Vec<u8>,
    pub config: LkjmcConfig,
    pub fleet: FleetSnapshot,
    pub asset_closure_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InstallPhase {
    Prepared,
    IdentityAndRoots,
    Secrets,
    Database,
    ReleaseAndAssets,
    Fenced,
    ServicePublished,
    Activating,
    Accepted,
    RecoveryBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InstallJournal {
    schema_version: u32,
    operation_id: Uuid,
    input_sha256: String,
    release_commit: String,
    release_manifest_sha256: String,
    configuration_sha256: String,
    asset_closure_sha256: String,
    state_directory: PathBuf,
    phase: InstallPhase,
    first_failure: Option<String>,
    receipt_sha256: Option<String>,
    fence_cleared: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInstallReceipt {
    pub schema_version: u32,
    pub result: InstallResult,
    pub operation_id: Uuid,
    pub release_commit: String,
    pub release_manifest_sha256: String,
    pub install_input_sha256: String,
    pub configuration_sha256: String,
    pub runtime_asset_closure_sha256: String,
    pub fleet_revision: u64,
    pub instance_ids: Vec<String>,
    pub velocity_instance_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallResult {
    Accepted,
    NoOp,
}

enum TargetState {
    Fresh,
    Resumable(InstallJournal),
    Accepted(InstallJournal),
}

impl HostInstallRequest {
    pub fn validate(&self) -> Result<()> {
        require_absolute_safe(&self.input_path, "first-install input path")?;
        require_hex(&self.input_sha256, 64, "first-install input SHA-256")
    }
}

impl InstallJournal {
    fn new(inspection: &InstallInspection, state_directory: PathBuf) -> Self {
        Self {
            schema_version: 2,
            operation_id: inspection.input.operation_id,
            input_sha256: inspection.input_sha256.clone(),
            release_commit: inspection.release.manifest.commit.clone(),
            release_manifest_sha256: inspection.release.manifest_sha256.clone(),
            configuration_sha256: inspection.input.configuration.sha256.clone(),
            asset_closure_sha256: inspection.asset_closure_sha256.clone(),
            state_directory,
            phase: InstallPhase::Prepared,
            first_failure: None,
            receipt_sha256: None,
            fence_cleared: false,
        }
    }

    fn validate(&self) -> Result<()> {
        if self.schema_version != 2 {
            return Err(OpsError::message(
                "unsupported first-install journal schema",
            ));
        }
        require_hex(&self.input_sha256, 64, "install journal input SHA-256")?;
        require_hex(&self.release_commit, 40, "install journal release commit")?;
        require_hex(
            &self.release_manifest_sha256,
            64,
            "install journal release manifest SHA-256",
        )?;
        require_hex(
            &self.configuration_sha256,
            64,
            "install journal configuration SHA-256",
        )?;
        require_hex(
            &self.asset_closure_sha256,
            64,
            "install journal runtime asset closure SHA-256",
        )?;
        require_absolute_safe(&self.state_directory, "install journal state directory")?;
        if self
            .first_failure
            .as_deref()
            .is_some_and(|value| value.is_empty() || value.len() > 1024 || value.contains('\n'))
        {
            return Err(OpsError::message(
                "install journal first failure is not bounded or secret-safe",
            ));
        }
        if let Some(receipt_sha256) = &self.receipt_sha256 {
            require_hex(receipt_sha256, 64, "install journal receipt SHA-256")?;
            if self.phase != InstallPhase::Accepted {
                return Err(OpsError::message(
                    "only an accepted install journal may bind a final receipt",
                ));
            }
        } else if self.phase == InstallPhase::Accepted {
            return Err(OpsError::message(
                "accepted install journal has no final receipt identity",
            ));
        }
        if self.fence_cleared && self.phase != InstallPhase::Accepted {
            return Err(OpsError::message(
                "only an accepted install journal may clear the deployment fence",
            ));
        }
        Ok(())
    }

    fn matches(&self, inspection: &InstallInspection, state_directory: &Path) -> Result<()> {
        self.validate()?;
        let expected = Self::new(inspection, state_directory.to_path_buf());
        if self.operation_id != expected.operation_id
            || self.input_sha256 != expected.input_sha256
            || self.release_commit != expected.release_commit
            || self.release_manifest_sha256 != expected.release_manifest_sha256
            || self.configuration_sha256 != expected.configuration_sha256
            || self.asset_closure_sha256 != expected.asset_closure_sha256
            || self.state_directory != expected.state_directory
        {
            return Err(OpsError::message(
                "first-install input differs from the durable operation journal",
            ));
        }
        Ok(())
    }

    fn record_failure(&mut self, error: &OpsError) {
        if self.first_failure.is_none() {
            let mut value = error.to_string().replace(['\n', '\r'], " ");
            value.truncate(1024);
            self.first_failure = Some(value);
        }
    }
}

fn write_install_journal(path: &Path, journal: &InstallJournal) -> Result<()> {
    journal.validate()?;
    let mut raw = serde_json::to_vec(journal)?;
    raw.push(b'\n');
    atomic_write(path, &raw, 0o600, 0, 0)
}

fn read_install_journal(path: &Path) -> Result<InstallJournal> {
    let raw = read_regular(
        path,
        "first-install journal",
        Some(0),
        Some(0),
        Some(0o600),
        MAX_CONTROL_FILE_BYTES,
    )?;
    let journal: InstallJournal = serde_json::from_slice(&raw)
        .map_err(|error| OpsError::context("invalid first-install journal", error))?;
    journal.validate()?;
    Ok(journal)
}

pub fn install(request: HostInstallRequest) -> Result<HostInstallReceipt> {
    crate::require_root()?;
    let first = inspect(&request)?;
    let layout = HostLayout::from_config(&first.config)?;
    preflight_substrate(&first)?;
    let _lock = DeploymentLock::acquire(&layout.lock, Path::new("/run"), 0, 0)?;
    let inspection = inspect(&request)?;
    let layout = HostLayout::from_config(&inspection.config)?;
    preflight_substrate(&inspection)?;
    let target = classify_target(&inspection, &layout)?;
    let mut journal = match target {
        TargetState::Fresh => prepare_journal(&inspection, &layout)?,
        TargetState::Resumable(journal) => journal,
        TargetState::Accepted(mut journal) => {
            finalize_accepted_fence(&inspection, &layout, &mut journal)?;
            verify_accepted_target(&inspection, &layout, &journal)?;
            return read_final_receipt(&inspection, &journal, InstallResult::NoOp);
        }
    };
    host_deploy::verify_running_ops(&inspection.release)?;
    execute_install(&inspection, &layout, &mut journal)
}

fn classify_target(inspection: &InstallInspection, layout: &HostLayout) -> Result<TargetState> {
    let state_root = layout.installation_state_root();
    let state_directory = layout.installation_state_directory(inspection.input.operation_id);
    match fs::symlink_metadata(&state_directory) {
        Ok(metadata) => {
            if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
                return Err(OpsError::message(
                    "first-install operation state path is ambiguous",
                ));
            }
            require_directory(
                &state_directory,
                "first-install operation state",
                Some(0),
                Some(0),
                Some(0o700),
            )?;
            let journal = read_install_journal(&state_directory.join("journal.json"))?;
            journal.matches(inspection, &state_directory)?;
            if journal.phase == InstallPhase::Accepted {
                Ok(TargetState::Accepted(journal))
            } else {
                Ok(TargetState::Resumable(journal))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            ensure_no_other_installation_state(&state_root)?;
            ensure_fresh_target(inspection, layout)?;
            Ok(TargetState::Fresh)
        }
        Err(error) => Err(OpsError::context(
            "cannot inspect first-install operation state",
            error,
        )),
    }
}

fn ensure_no_other_installation_state(state_root: &Path) -> Result<()> {
    match fs::symlink_metadata(state_root) {
        Ok(metadata) => {
            if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
                return Err(OpsError::message("first-install state root is ambiguous"));
            }
            require_directory(
                state_root,
                "first-install state root",
                Some(0),
                Some(0),
                Some(0o700),
            )?;
            let mut entries = fs::read_dir(state_root).map_err(|error| {
                OpsError::context("cannot enumerate first-install state", error)
            })?;
            if entries
                .next()
                .transpose()
                .map_err(|error| OpsError::context("cannot read first-install state", error))?
                .is_some()
            {
                return Err(OpsError::message(
                    "another first-install operation already owns this target",
                ));
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(OpsError::context(
            "cannot inspect first-install state root",
            error,
        )),
    }
}

fn ensure_fresh_target(inspection: &InstallInspection, layout: &HostLayout) -> Result<()> {
    for (path, label) in [
        (Path::new(INSTALL_ROOT), "installation root"),
        (Path::new(CONFIG_ROOT), "configuration root"),
        (Path::new(DATA_ROOT), "data root"),
        (Path::new(LOG_ROOT), "log root"),
        (Path::new(BACKUP_ROOT), "backup root"),
        (&layout.unit, "systemd unit"),
        (&layout.fence_dropin, "systemd fence drop-in"),
        (&layout.fence, "deployment fence"),
        (&layout.permit, "deployment start permit"),
    ] {
        require_absent(path, label)?;
    }
    ensure_systemd_service_absent()?;
    ensure_no_existing_daemon_process()?;
    if lookup_getent("group", SERVICE_GROUP)?.is_some()
        || lookup_getent("passwd", SERVICE_USER)?.is_some()
    {
        return Err(OpsError::message(
            "first-install service user or group already exists and will not be adopted",
        ));
    }
    if lookup_getent("group", &inspection.input.service.gid.to_string())?.is_some()
        || lookup_getent("passwd", &inspection.input.service.uid.to_string())?.is_some()
    {
        return Err(OpsError::message(
            "first-install service numeric identity already belongs to another account and will not be adopted",
        ));
    }
    match database_ownership(inspection)? {
        DatabaseOwnership::Absent => Ok(()),
        DatabaseOwnership::Owned | DatabaseOwnership::RoleOwnedOnly => Err(OpsError::message(
            "first-install PostgreSQL target is already marked by this operation without a durable journal",
        )),
        DatabaseOwnership::Conflict => Err(OpsError::message(
            "first-install PostgreSQL role or database already exists and is not owned by this operation",
        )),
    }
}

fn ensure_systemd_service_absent() -> Result<()> {
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from(host_deploy::SYSTEMCTL),
        arguments: vec![
            "show".to_string(),
            "--property=LoadState".to_string(),
            "--value".to_string(),
            host_deploy::SERVICE.to_string(),
        ],
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(30),
        max_output_bytes: 64 * 1024,
    })?;
    let state = std::str::from_utf8(&output.stdout)
        .map_err(|_| OpsError::message("systemd load-state output is not UTF-8"))?
        .trim();
    if output.status != 0 || state != "not-found" {
        return Err(OpsError::message(
            "first-install systemd service already exists and will not be adopted",
        ));
    }
    Ok(())
}

fn ensure_no_existing_daemon_process() -> Result<()> {
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from("/usr/bin/pgrep"),
        arguments: vec!["-x".to_string(), "lkjmc-daemon".to_string()],
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(15),
        max_output_bytes: 16 * 1024,
    })?;
    match output.status {
        1 => Ok(()),
        0 => Err(OpsError::message(
            "first-install found an existing lkjmc-daemon process and will not adopt it",
        )),
        _ => Err(OpsError::message(
            "cannot determine whether an existing lkjmc-daemon process is present",
        )),
    }
}

fn require_absent(path: &Path, label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(OpsError::message(format!(
            "first-install {label} already exists and will not be adopted: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(OpsError::context(
            &format!("cannot inspect first-install {label}"),
            error,
        )),
    }
}

fn prepare_journal(inspection: &InstallInspection, layout: &HostLayout) -> Result<InstallJournal> {
    let state_root = layout.installation_state_root();
    validate_ancestry(&state_root, Path::new("/"), 0)?;
    let state_parent = state_root
        .parent()
        .ok_or_else(|| OpsError::message("first-install state root has no parent"))?;
    require_directory(
        state_parent,
        "first-install private state parent",
        Some(0),
        Some(0),
        Some(0o700),
    )?;
    match fs::symlink_metadata(&state_root) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            create_directory(&state_root, 0o700, 0, 0)?;
            sync_directory(state_parent)?;
        }
        Err(error) => {
            return Err(OpsError::context(
                "cannot inspect first-install state root before journal creation",
                error,
            ));
        }
    }
    require_directory(
        &state_root,
        "first-install state root",
        Some(0),
        Some(0),
        Some(0o700),
    )?;
    let state_directory = layout.installation_state_directory(inspection.input.operation_id);
    create_directory(&state_directory, 0o700, 0, 0)?;
    sync_directory(&state_root)?;
    let journal = InstallJournal::new(inspection, state_directory.clone());
    write_install_journal(&state_directory.join("journal.json"), &journal)?;
    sync_directory(&state_directory)?;
    Ok(journal)
}

fn preflight_substrate(inspection: &InstallInspection) -> Result<()> {
    if fs::read_to_string("/proc/1/comm")
        .map_err(|error| OpsError::context("cannot inspect PID 1", error))?
        .trim()
        != "systemd"
    {
        return Err(OpsError::message("first-install requires systemd as PID 1"));
    }
    require_directory(
        Path::new("/run/systemd/system"),
        "systemd runtime directory",
        Some(0),
        None,
        None,
    )?;
    require_tool(Path::new(host_deploy::SYSTEMCTL), &["--version"], "systemd")?;
    require_java_21()?;
    require_postgres_14()?;
    for path in [
        "/usr/lib/postgresql/14/bin/pg_dump",
        "/usr/lib/postgresql/14/bin/pg_restore",
    ] {
        require_tool(
            Path::new(path),
            &["--version"],
            "PostgreSQL 14 backup utility",
        )?;
    }
    for path in [
        "/usr/bin/getent",
        "/usr/sbin/groupadd",
        "/usr/sbin/useradd",
        "/usr/sbin/runuser",
    ] {
        crate::process::trusted_executable(Path::new(path))?;
    }
    for path in ["/opt", "/etc", "/var/lib", "/var/log", "/var/backups"] {
        validate_first_install_root_parent(Path::new(path))?;
    }
    validate_capacity(&inspection.input.capacity)?;
    preflight_listener_conflicts(inspection)?;
    let output = psql_admin(
        inspection,
        "select current_setting('server_version_num'), current_setting('port');",
        30,
    )?;
    let postgres = std::str::from_utf8(&output)
        .map_err(|_| OpsError::message("PostgreSQL preflight output is not UTF-8"))?
        .trim_end()
        .split('|')
        .collect::<Vec<_>>();
    let supported_server = postgres
        .first()
        .and_then(|value| value.parse::<u32>().ok())
        .is_some_and(|value| (140_000..150_000).contains(&value));
    let configured_port = postgres.get(1).and_then(|value| value.parse::<u16>().ok())
        == Some(inspection.config.database.port);
    if postgres.len() != 2 || !supported_server || !configured_port {
        return Err(OpsError::message(
            "local PostgreSQL server version or configured port differs from the supported contract",
        ));
    }
    Ok(())
}

fn validate_first_install_root_parent(path: &Path) -> Result<()> {
    let metadata = require_directory(path, "first-install root parent", Some(0), None, None)?;
    let mode = metadata.mode() & 0o7777;
    if mode & 0o022 == 0 {
        return Ok(());
    }
    if accepts_standard_syslog_log_parent(path, metadata.uid(), metadata.gid(), mode)? {
        return Ok(());
    }
    Err(OpsError::message(format!(
        "first-install root parent is group/other writable: {}",
        path.display()
    )))
}

fn accepts_standard_syslog_log_parent(path: &Path, uid: u32, gid: u32, mode: u32) -> Result<bool> {
    Ok(is_standard_syslog_log_parent(
        path,
        uid,
        gid,
        mode,
        syslog_group_gid()?,
    ))
}

fn is_standard_syslog_log_parent(
    path: &Path,
    uid: u32,
    gid: u32,
    mode: u32,
    syslog_gid: Option<u32>,
) -> bool {
    path == Path::new("/var/log") && uid == 0 && mode == 0o775 && syslog_gid == Some(gid)
}

fn syslog_group_gid() -> Result<Option<u32>> {
    let Some(record) = lookup_getent("group", "syslog")? else {
        return Ok(None);
    };
    parse_group_gid(&record, "syslog").map(Some)
}

fn parse_group_gid(record: &str, expected_name: &str) -> Result<u32> {
    let fields = record.split(':').collect::<Vec<_>>();
    if fields.len() != 4 || fields[0] != expected_name || fields[1].is_empty() {
        return Err(OpsError::message("system group lookup output is malformed"));
    }
    fields[2]
        .parse::<u32>()
        .map_err(|_| OpsError::message("system group lookup output is malformed"))
}

fn require_tool(path: &Path, arguments: &[&str], label: &str) -> Result<()> {
    let output = run_bounded(&CommandSpec {
        executable: path.to_path_buf(),
        arguments: arguments.iter().map(|value| (*value).to_string()).collect(),
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(30),
        max_output_bytes: 64 * 1024,
    })?;
    if output.status != 0 {
        return Err(OpsError::message(format!(
            "required first-install substrate tool is unavailable: {label}"
        )));
    }
    Ok(())
}

fn require_java_21() -> Result<()> {
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from("/usr/bin/java"),
        arguments: vec!["-version".to_string()],
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(30),
        max_output_bytes: 64 * 1024,
    })?;
    if output.status != 0
        || first_decimal(&output.stdout, &output.stderr).is_none_or(|value| value < 21)
    {
        return Err(OpsError::message(
            "first-install requires a supported Java 21-or-newer runtime",
        ));
    }
    Ok(())
}

fn require_postgres_14() -> Result<()> {
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from("/usr/bin/psql"),
        arguments: vec!["--version".to_string()],
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(30),
        max_output_bytes: 64 * 1024,
    })?;
    if output.status != 0 || first_decimal(&output.stdout, &output.stderr) != Some(14) {
        return Err(OpsError::message(
            "first-install requires PostgreSQL 14 client tools for the packaged backup contract",
        ));
    }
    Ok(())
}

fn first_decimal(stdout: &[u8], stderr: &[u8]) -> Option<u32> {
    let mut combined = Vec::with_capacity(stdout.len() + stderr.len());
    combined.extend_from_slice(stdout);
    combined.extend_from_slice(stderr);
    let text = std::str::from_utf8(&combined).ok()?;
    text.split(|character: char| !character.is_ascii_digit())
        .find(|value| !value.is_empty())?
        .parse()
        .ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatabaseOwnership {
    Absent,
    RoleOwnedOnly,
    Owned,
    Conflict,
}

fn database_ownership(inspection: &InstallInspection) -> Result<DatabaseOwnership> {
    let role = sql_literal(&inspection.input.postgres.role)?;
    let database = sql_literal(&inspection.input.postgres.database)?;
    let query = format!(
        "select case when exists (select 1 from pg_roles where rolname = {role}) then '1' else '0' end, \
         coalesce((select shobj_description(oid, 'pg_authid') from pg_roles where rolname = {role}), ''), \
         case when exists (select 1 from pg_database where datname = {database}) then '1' else '0' end, \
         coalesce((select shobj_description(oid, 'pg_database') from pg_database where datname = {database}), '');"
    );
    let output = psql_admin(inspection, &query, 30)?;
    let text = std::str::from_utf8(&output)
        .map_err(|_| OpsError::message("PostgreSQL ownership preflight output is not UTF-8"))?
        .trim_end();
    let fields = text.split('|').collect::<Vec<_>>();
    if fields.len() != 4 || fields.iter().any(|field| field.contains('\n')) {
        return Err(OpsError::message(
            "PostgreSQL ownership preflight output is malformed",
        ));
    }
    match (fields[0], fields[2]) {
        ("0", "0") if fields[1].is_empty() && fields[3].is_empty() => Ok(DatabaseOwnership::Absent),
        ("1", "0") if fields[1] == install_marker(inspection) && fields[3].is_empty() => {
            Ok(DatabaseOwnership::RoleOwnedOnly)
        }
        ("1", "1")
            if fields[1] == install_marker(inspection)
                && fields[3] == install_marker(inspection) =>
        {
            Ok(DatabaseOwnership::Owned)
        }
        _ => Ok(DatabaseOwnership::Conflict),
    }
}

fn install_marker(inspection: &InstallInspection) -> String {
    format!(
        "lkjmc-install-v1:{}:{}",
        inspection.input.operation_id, inspection.input_sha256
    )
}

fn psql_admin(inspection: &InstallInspection, sql: &str, timeout_seconds: u64) -> Result<Vec<u8>> {
    if sql.len() > MAX_CONTROL_FILE_BYTES as usize || sql.bytes().any(|value| value == 0) {
        return Err(OpsError::message(
            "PostgreSQL administrative input is oversized or invalid",
        ));
    }
    let arguments = vec![
        "--user".to_string(),
        inspection.input.postgres.administrative_user.clone(),
        "--".to_string(),
        "/usr/bin/psql".to_string(),
        "--no-psqlrc".to_string(),
        "--no-align".to_string(),
        "--tuples-only".to_string(),
        "--quiet".to_string(),
        "--set".to_string(),
        "ON_ERROR_STOP=1".to_string(),
        "--field-separator".to_string(),
        "|".to_string(),
        "--host".to_string(),
        inspection
            .input
            .postgres
            .socket_directory
            .display()
            .to_string(),
        "--username".to_string(),
        inspection.input.postgres.administrative_user.clone(),
        "--dbname".to_string(),
        "postgres".to_string(),
    ];
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from("/usr/sbin/runuser"),
        arguments,
        environment: BTreeMap::new(),
        stdin: sql.as_bytes().to_vec(),
        timeout: Duration::from_secs(timeout_seconds),
        max_output_bytes: 256 * 1024,
    })?;
    if output.status != 0 {
        return Err(OpsError::message(
            "local PostgreSQL administrative operation failed",
        ));
    }
    Ok(output.stdout)
}

fn sql_literal(value: &str) -> Result<String> {
    if value.bytes().any(|byte| byte == 0 || byte == b'\'') {
        return Err(OpsError::message("unsafe SQL literal value"));
    }
    Ok(format!("'{value}'"))
}

fn sql_identifier(value: &str) -> Result<String> {
    validate_postgres_identifier(value, "PostgreSQL identifier")?;
    Ok(format!("\"{value}\""))
}

fn lookup_getent(database: &str, key: &str) -> Result<Option<String>> {
    if !matches!(database, "group" | "passwd") || key.is_empty() || key.len() > 63 {
        return Err(OpsError::message("unsafe identity lookup"));
    }
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from("/usr/bin/getent"),
        arguments: vec![database.to_string(), key.to_string()],
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(15),
        max_output_bytes: 16 * 1024,
    })?;
    match output.status {
        0 => {
            let text = std::str::from_utf8(&output.stdout)
                .map_err(|_| OpsError::message("identity lookup output is not UTF-8"))?;
            let value = text.trim_end();
            if value.is_empty() || value.contains('\n') || value.len() > 4096 {
                return Err(OpsError::message("identity lookup output is malformed"));
            }
            Ok(Some(value.to_string()))
        }
        2 => Ok(None),
        _ => Err(OpsError::message("identity lookup failed")),
    }
}

fn validate_capacity(capacity: &CapacityContract) -> Result<()> {
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from("/usr/bin/df"),
        arguments: vec!["-Pk".to_string(), "/var".to_string()],
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(15),
        max_output_bytes: 64 * 1024,
    })?;
    if output.status != 0 {
        return Err(OpsError::message(
            "first-install storage capacity observation failed",
        ));
    }
    let output = std::str::from_utf8(&output.stdout)
        .map_err(|_| OpsError::message("first-install storage capacity output is not UTF-8"))?;
    let line = output
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| OpsError::message("first-install storage capacity output is empty"))?;
    let available_kib = line
        .split_whitespace()
        .nth(3)
        .ok_or_else(|| OpsError::message("first-install storage capacity output is malformed"))?
        .parse::<u64>()
        .map_err(|_| OpsError::message("first-install storage capacity value is invalid"))?;
    if available_kib / 1024 < capacity.minimum_free_mib {
        return Err(OpsError::message(
            "first-install storage capacity is below the declared minimum",
        ));
    }
    let memory_kib = proc_value("/proc/meminfo", "MemAvailable:")?;
    if memory_kib / 1024 < capacity.minimum_memory_mib {
        return Err(OpsError::message(
            "first-install memory capacity is below the declared minimum",
        ));
    }
    let maximum_processes = proc_integer("/proc/sys/kernel/pid_max")?;
    if maximum_processes < u64::from(capacity.minimum_processes) {
        return Err(OpsError::message(
            "first-install process capacity is below the declared minimum",
        ));
    }
    let maximum_open_files = proc_integer("/proc/sys/fs/file-max")?;
    if maximum_open_files < capacity.minimum_open_files {
        return Err(OpsError::message(
            "first-install file-descriptor capacity is below the declared minimum",
        ));
    }
    Ok(())
}

fn proc_value(path: &str, prefix: &str) -> Result<u64> {
    let text = fs::read_to_string(path)
        .map_err(|error| OpsError::context("cannot read first-install capacity state", error))?;
    text.lines()
        .find_map(|line| line.strip_prefix(prefix))
        .and_then(|value| value.split_whitespace().next())
        .ok_or_else(|| OpsError::message("first-install capacity state is malformed"))?
        .parse::<u64>()
        .map_err(|_| OpsError::message("first-install capacity value is invalid"))
}

fn proc_integer(path: &str) -> Result<u64> {
    fs::read_to_string(path)
        .map_err(|error| OpsError::context("cannot read first-install capacity state", error))?
        .trim()
        .parse::<u64>()
        .map_err(|_| OpsError::message("first-install capacity value is invalid"))
}

fn preflight_listener_conflicts(inspection: &InstallInspection) -> Result<()> {
    let mut listeners = Vec::new();
    for listener in &inspection.config.network.listeners {
        let address = format!("{}:{}", listener.bind_host, listener.port)
            .parse::<std::net::SocketAddr>()
            .map_err(|_| OpsError::message("configured listener address is invalid"))?;
        let bound = std::net::TcpListener::bind(address).map_err(|_| {
            OpsError::message(format!(
                "configured listener is already unavailable: {}:{}",
                listener.bind_host, listener.port
            ))
        })?;
        listeners.push(bound);
    }
    drop(listeners);
    Ok(())
}

fn execute_install(
    inspection: &InstallInspection,
    layout: &HostLayout,
    journal: &mut InstallJournal,
) -> Result<HostInstallReceipt> {
    if journal.phase == InstallPhase::RecoveryBlocked {
        return Err(OpsError::message(
            "first-install journal is recovery-blocked; inspect its first causal failure before manual recovery",
        ));
    }
    let journal_path = journal.state_directory.join("journal.json");
    let result = (|| {
        advance_stage(
            journal,
            InstallPhase::IdentityAndRoots,
            &journal_path,
            || ensure_identity_and_roots(inspection, false),
            || ensure_identity_and_roots(inspection, true),
        )?;
        advance_stage(
            journal,
            InstallPhase::Secrets,
            &journal_path,
            || ensure_secrets(inspection, false),
            || ensure_secrets(inspection, true),
        )?;
        advance_stage(
            journal,
            InstallPhase::Database,
            &journal_path,
            || ensure_database(inspection, false),
            || ensure_database(inspection, true),
        )?;
        advance_stage(
            journal,
            InstallPhase::ReleaseAndAssets,
            &journal_path,
            || ensure_release_and_assets(inspection, layout, false),
            || ensure_release_and_assets(inspection, layout, true),
        )?;
        advance_stage(
            journal,
            InstallPhase::Fenced,
            &journal_path,
            || ensure_install_fence(inspection, layout, false),
            || ensure_install_fence(inspection, layout, true),
        )?;
        advance_stage(
            journal,
            InstallPhase::ServicePublished,
            &journal_path,
            || ensure_service_publication(inspection, layout, false),
            || ensure_service_publication(inspection, layout, true),
        )?;
        advance_stage(
            journal,
            InstallPhase::Activating,
            &journal_path,
            || activate_service(inspection, layout, false),
            || activate_service(inspection, layout, false),
        )?;
        finish_acceptance(inspection, layout, journal)
    })();
    if let Err(error) = result {
        journal.record_failure(&error);
        if matches!(
            journal.phase,
            InstallPhase::ServicePublished | InstallPhase::Activating
        ) {
            let _ = host_deploy::stop_service();
        }
        let _ = write_install_journal(&journal_path, journal);
        return Err(error);
    }
    result
}

fn advance_stage(
    journal: &mut InstallJournal,
    target: InstallPhase,
    journal_path: &Path,
    create: impl FnOnce() -> Result<()>,
    verify: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let current = phase_rank(journal.phase)?;
    let target_rank = phase_rank(target)?;
    if current < target_rank {
        create()?;
        journal.phase = target;
        write_install_journal(journal_path, journal)
    } else {
        verify()
    }
}

fn phase_rank(phase: InstallPhase) -> Result<u8> {
    match phase {
        InstallPhase::Prepared => Ok(0),
        InstallPhase::IdentityAndRoots => Ok(1),
        InstallPhase::Secrets => Ok(2),
        InstallPhase::Database => Ok(3),
        InstallPhase::ReleaseAndAssets => Ok(4),
        InstallPhase::Fenced => Ok(5),
        InstallPhase::ServicePublished => Ok(6),
        InstallPhase::Activating => Ok(7),
        InstallPhase::Accepted => Ok(8),
        InstallPhase::RecoveryBlocked => Err(OpsError::message(
            "first-install journal is recovery-blocked",
        )),
    }
}

fn ensure_identity_and_roots(inspection: &InstallInspection, verify_only: bool) -> Result<()> {
    ensure_service_identity(&inspection.input.service, verify_only)?;
    let service = &inspection.input.service;
    let roots = [
        (
            PathBuf::from(INSTALL_ROOT),
            0o750,
            0,
            service.gid,
            "installation root",
        ),
        (
            PathBuf::from(INSTALL_ROOT).join("releases"),
            0o750,
            0,
            service.gid,
            "release root",
        ),
        (
            PathBuf::from(RUNTIME_ASSET_ROOT),
            0o750,
            0,
            service.gid,
            "runtime asset root",
        ),
        (
            PathBuf::from(JAR_ROOT),
            0o750,
            0,
            service.gid,
            "immutable jar root",
        ),
        (
            PathBuf::from(CONFIG_ROOT),
            0o750,
            0,
            service.gid,
            "configuration root",
        ),
        (
            PathBuf::from(DATA_ROOT),
            0o750,
            service.uid,
            service.gid,
            "data root",
        ),
        (
            PathBuf::from(DATA_ROOT).join("instances"),
            0o750,
            service.uid,
            service.gid,
            "managed instances root",
        ),
        (
            PathBuf::from(DATA_ROOT).join("private"),
            0o700,
            service.uid,
            service.gid,
            "private data root",
        ),
        (
            PathBuf::from(DATA_ROOT).join("private/plugin-credentials"),
            0o700,
            service.uid,
            service.gid,
            "plugin credential root",
        ),
        (
            PathBuf::from(LOG_ROOT),
            0o750,
            service.uid,
            service.gid,
            "log root",
        ),
        (PathBuf::from(BACKUP_ROOT), 0o700, 0, 0, "backup root"),
    ];
    for (path, mode, uid, gid, label) in roots {
        ensure_directory(&path, mode, uid, gid, label, verify_only)?;
    }
    let fleet = &inspection.fleet;
    for instance in fleet.instances() {
        let root = fleet.instance_root(instance.id.as_str());
        ensure_directory(
            &root,
            0o750,
            service.uid,
            service.gid,
            "managed instance root",
            verify_only,
        )?;
    }
    for plugin in fleet.plugin_targets() {
        let parent = plugin
            .destination
            .parent()
            .ok_or_else(|| OpsError::message("managed plugin destination has no parent"))?;
        ensure_directory(
            parent,
            0o750,
            service.uid,
            service.gid,
            "managed plugin root",
            verify_only,
        )?;
    }
    ensure_copied_file(
        &inspection.config_bytes,
        Path::new(CONFIG_ROOT).join("lkjmc.json").as_path(),
        0o640,
        0,
        service.gid,
        "canonical configuration",
        verify_only,
    )?;
    ensure_copied_file(
        b"# lkjmc uses config-bound secret files\n",
        Path::new(CONFIG_ROOT).join("daemon.env").as_path(),
        0o640,
        0,
        service.gid,
        "daemon environment file",
        verify_only,
    )?;
    Ok(())
}

fn ensure_service_identity(service: &ServiceContract, verify_only: bool) -> Result<()> {
    match lookup_getent("group", &service.group)? {
        Some(value) => validate_group_record(&value, service)?,
        None if verify_only => {
            return Err(OpsError::message(
                "first-install service group disappeared after its durable phase",
            ));
        }
        None => {
            let gid = service.gid.to_string();
            run_identity_tool(
                "/usr/sbin/groupadd",
                &["--system", "--gid", &gid, &service.group],
                "create first-install service group",
            )?;
            let value = lookup_getent("group", &service.group)?.ok_or_else(|| {
                OpsError::message("created first-install service group is not observable")
            })?;
            validate_group_record(&value, service)?;
        }
    }
    match lookup_getent("passwd", &service.user)? {
        Some(value) => validate_passwd_record(&value, service)?,
        None if verify_only => {
            return Err(OpsError::message(
                "first-install service user disappeared after its durable phase",
            ));
        }
        None => {
            let uid = service.uid.to_string();
            run_identity_tool(
                "/usr/sbin/useradd",
                &[
                    "--system",
                    "--uid",
                    &uid,
                    "--gid",
                    &service.group,
                    "--home-dir",
                    SERVICE_HOME,
                    "--shell",
                    SERVICE_SHELL,
                    "--no-create-home",
                    &service.user,
                ],
                "create first-install service user",
            )?;
            let value = lookup_getent("passwd", &service.user)?.ok_or_else(|| {
                OpsError::message("created first-install service user is not observable")
            })?;
            validate_passwd_record(&value, service)?;
        }
    }
    Ok(())
}

fn validate_group_record(record: &str, service: &ServiceContract) -> Result<()> {
    let fields = record.split(':').collect::<Vec<_>>();
    if fields.len() != 4
        || fields[0] != service.group
        || fields[2].parse::<u32>().ok() != Some(service.gid)
    {
        return Err(OpsError::message(
            "existing first-install service group identity differs",
        ));
    }
    Ok(())
}

fn validate_passwd_record(record: &str, service: &ServiceContract) -> Result<()> {
    let fields = record.split(':').collect::<Vec<_>>();
    if fields.len() != 7
        || fields[0] != service.user
        || fields[2].parse::<u32>().ok() != Some(service.uid)
        || fields[3].parse::<u32>().ok() != Some(service.gid)
        || fields[5] != SERVICE_HOME
        || fields[6] != SERVICE_SHELL
    {
        return Err(OpsError::message(
            "existing first-install service user identity differs",
        ));
    }
    Ok(())
}

fn run_identity_tool(path: &str, arguments: &[&str], label: &str) -> Result<()> {
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from(path),
        arguments: arguments.iter().map(|value| (*value).to_string()).collect(),
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(30),
        max_output_bytes: 64 * 1024,
    })?;
    if output.status != 0 {
        return Err(OpsError::message(format!("{label} failed")));
    }
    Ok(())
}

fn ensure_directory(
    path: &Path,
    mode: u32,
    uid: u32,
    gid: u32,
    label: &str,
    verify_only: bool,
) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            require_directory(path, label, Some(uid), Some(gid), Some(mode))?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !verify_only => {
            let parent = path
                .parent()
                .ok_or_else(|| OpsError::message(format!("{label} has no parent")))?;
            require_directory(parent, "first-install root parent", None, None, None)?;
            create_directory(path, mode, uid, gid)?;
            sync_directory(parent)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(OpsError::message(
            format!("{label} disappeared after its durable phase"),
        )),
        Err(error) => Err(OpsError::context(&format!("cannot inspect {label}"), error)),
    }
}

fn ensure_copied_file(
    expected: &[u8],
    path: &Path,
    mode: u32,
    uid: u32,
    gid: u32,
    label: &str,
    verify_only: bool,
) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            let actual = read_regular(
                path,
                label,
                Some(uid),
                Some(gid),
                Some(mode),
                MAX_CONTROL_FILE_BYTES,
            )?;
            if actual != expected {
                return Err(OpsError::message(format!(
                    "{label} differs from its anchored bytes"
                )));
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !verify_only => {
            atomic_write(path, expected, mode, uid, gid)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(OpsError::message(
            format!("{label} disappeared after its durable phase"),
        )),
        Err(error) => Err(OpsError::context(&format!("cannot inspect {label}"), error)),
    }
}

fn ensure_secrets(inspection: &InstallInspection, verify_only: bool) -> Result<()> {
    let service = &inspection.input.service;
    for (path, label) in [
        (Path::new(DATABASE_SECRET), "database secret"),
        (Path::new(FORWARDING_SECRET), "forwarding secret"),
        (Path::new(HTTP_TOKEN), "daemon HTTP token"),
    ] {
        ensure_generated_secret(path, service.uid, service.gid, label, verify_only)?;
    }
    for target in inspection.fleet.credential_targets() {
        ensure_generated_secret(
            &target.path,
            service.uid,
            service.gid,
            "plugin heartbeat credential",
            verify_only,
        )?;
    }
    Ok(())
}

fn ensure_generated_secret(
    path: &Path,
    uid: u32,
    gid: u32,
    label: &str,
    verify_only: bool,
) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            let bytes = read_regular(path, label, Some(uid), Some(gid), Some(0o600), 4096)?;
            if bytes.len() < 32 || bytes.len() > 4096 || !bytes.ends_with(b"\n") {
                return Err(OpsError::message(format!("{label} is empty or malformed")));
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !verify_only => {
            let mut secret = [0_u8; 32];
            std::fs::File::open("/dev/urandom")
                .and_then(|mut file| std::io::Read::read_exact(&mut file, &mut secret))
                .map_err(|error| {
                    OpsError::context("cannot read operating-system randomness", error)
                })?;
            let mut bytes = hex(&secret).into_bytes();
            bytes.push(b'\n');
            atomic_write(path, &bytes, 0o600, uid, gid)?;
            let _ = read_regular(path, label, Some(uid), Some(gid), Some(0o600), 4096)?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(OpsError::message(
            format!("{label} disappeared after its durable phase"),
        )),
        Err(error) => Err(OpsError::context(&format!("cannot inspect {label}"), error)),
    }
}

fn hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(TABLE[(byte >> 4) as usize] as char);
        output.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    output
}

fn ensure_database(inspection: &InstallInspection, verify_only: bool) -> Result<()> {
    match database_ownership(inspection)? {
        DatabaseOwnership::Owned => {}
        DatabaseOwnership::RoleOwnedOnly if !verify_only => create_database(inspection)?,
        DatabaseOwnership::Absent if !verify_only => {
            create_role(inspection)?;
            match database_ownership(inspection)? {
                DatabaseOwnership::RoleOwnedOnly => create_database(inspection)?,
                DatabaseOwnership::Owned => {}
                _ => {
                    return Err(OpsError::message(
                        "PostgreSQL role creation did not leave the expected owned state",
                    ));
                }
            }
        }
        DatabaseOwnership::Absent | DatabaseOwnership::RoleOwnedOnly => {
            return Err(OpsError::message(
                "PostgreSQL target disappeared after its durable phase",
            ));
        }
        DatabaseOwnership::Conflict => {
            return Err(OpsError::message(
                "PostgreSQL role or database is not owned by this first-install operation",
            ));
        }
    }
    if database_ownership(inspection)? != DatabaseOwnership::Owned {
        return Err(OpsError::message(
            "PostgreSQL initialization did not reach the owned role and database state",
        ));
    }
    let migrations = if verify_only {
        let mut connection = crate::database::connect(&inspection.config, None)?;
        crate::database::migration_marker(&mut connection.client)?
    } else {
        crate::database::apply_migrations(&inspection.config)?
    };
    if migrations.is_empty() {
        return Err(OpsError::message(
            "PostgreSQL migration ledger is unexpectedly empty after initialization",
        ));
    }
    ensure_plugin_heartbeat_credentials(inspection, verify_only)?;
    Ok(())
}

fn ensure_plugin_heartbeat_credentials(
    inspection: &InstallInspection,
    verify_only: bool,
) -> Result<()> {
    let service = &inspection.input.service;
    let targets = inspection.fleet.credential_targets();
    let bindings = targets
        .iter()
        .map(|target| {
            Ok((
                target,
                lkjmc_core::security::token_hash(&read_plugin_heartbeat_secret(
                    &target.path,
                    service,
                )?),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut connection = crate::database::connect(&inspection.config, None)?;
    let mut transaction = connection
        .client
        .transaction()
        .map_err(|error| OpsError::context("cannot begin plugin credential transaction", error))?;
    let actor_name = inspection.input.operation_id.to_string();
    for (target, token_hash) in bindings {
        match lkjmc_store::daemon_token::find_active(&mut transaction, &token_hash).map_err(
            |error| OpsError::context("cannot inspect plugin heartbeat credential", error),
        )? {
            Some(record) if plugin_heartbeat_record_matches(&record, target) => continue,
            Some(_) => {
                return Err(OpsError::message(format!(
                    "plugin heartbeat credential is bound to an unexpected principal: {}",
                    target.instance_id.as_str()
                )));
            }
            None if lkjmc_store::daemon_token::token_hash_exists(&mut transaction, &token_hash)
                .map_err(|error| {
                    OpsError::context("cannot inspect plugin heartbeat credential", error)
                })? =>
            {
                return Err(OpsError::message(format!(
                    "plugin heartbeat credential is expired or revoked and cannot be reactivated automatically: {}",
                    target.instance_id.as_str()
                )));
            }
            None if verify_only => {
                return Err(OpsError::message(format!(
                    "plugin heartbeat credential is missing its active database binding: {}",
                    target.instance_id.as_str()
                )));
            }
            None => {}
        }
        let scopes = vec![PLUGIN_HEARTBEAT_SCOPE.to_string()];
        let credential_id = Uuid::new_v4();
        let credential_id_text = credential_id.to_string();
        lkjmc_store::daemon_token::insert(
            &mut transaction,
            credential_id,
            &token_hash,
            target.surface,
            "instance",
            target.instance_id.as_str(),
            &scopes,
            PLUGIN_HEARTBEAT_EXPIRY_SECONDS,
        )
        .map_err(|error| OpsError::context("cannot create plugin heartbeat credential", error))?;
        lkjmc_store::audit::insert(
            &mut transaction,
            lkjmc_store::audit::NewAuditEvent {
                id: Uuid::new_v4(),
                actor_kind: "installer",
                actor_name: &actor_name,
                action: "host.install.plugin-heartbeat-credential.seed",
                target_kind: "credential",
                target_id: &credential_id_text,
                result: "succeeded",
            },
        )
        .map_err(|error| OpsError::context("cannot audit plugin heartbeat credential", error))?;
    }
    transaction
        .commit()
        .map_err(|error| OpsError::context("cannot commit plugin credential transaction", error))
}

fn read_plugin_heartbeat_secret(path: &Path, service: &ServiceContract) -> Result<String> {
    let raw = read_regular(
        path,
        "plugin heartbeat credential",
        Some(service.uid),
        Some(service.gid),
        Some(0o600),
        4096,
    )?;
    parse_plugin_heartbeat_secret(&raw).map(ToString::to_string)
}

fn parse_plugin_heartbeat_secret(raw: &[u8]) -> Result<&str> {
    let value = std::str::from_utf8(raw)
        .map_err(|_| OpsError::message("plugin heartbeat credential is not UTF-8"))?
        .strip_suffix('\n')
        .ok_or_else(|| OpsError::message("plugin heartbeat credential is malformed"))?;
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(OpsError::message(
            "plugin heartbeat credential is malformed",
        ));
    }
    Ok(value)
}

fn plugin_heartbeat_record_matches(
    record: &lkjmc_store::daemon_token::DaemonTokenRecord,
    target: &crate::fleet::CredentialTarget,
) -> bool {
    record.surface == target.surface
        && record.principal_kind == "instance"
        && record.principal_id == target.instance_id.as_str()
        && record.scopes.len() == 1
        && record.scopes[0] == PLUGIN_HEARTBEAT_SCOPE
}

fn create_role(inspection: &InstallInspection) -> Result<()> {
    let password = read_database_password(inspection)?;
    let role = sql_identifier(&inspection.input.postgres.role)?;
    let password = sql_literal(&password)?;
    let marker = sql_literal(&install_marker(inspection))?;
    let sql = format!(
        "create role {role} login password {password} nosuperuser nocreatedb nocreaterole noinherit; \
         comment on role {role} is {marker};"
    );
    let _ = psql_admin(inspection, &sql, 60)?;
    Ok(())
}

fn create_database(inspection: &InstallInspection) -> Result<()> {
    let database = sql_identifier(&inspection.input.postgres.database)?;
    let role = sql_identifier(&inspection.input.postgres.role)?;
    let marker = sql_literal(&install_marker(inspection))?;
    let sql = format!(
        "create database {database} owner {role} template template0 encoding 'UTF8'; \
         revoke all on database {database} from public; \
         comment on database {database} is {marker};"
    );
    let _ = psql_admin(inspection, &sql, 60)?;
    Ok(())
}

fn read_database_password(inspection: &InstallInspection) -> Result<String> {
    let raw = read_regular(
        Path::new(DATABASE_SECRET),
        "database secret",
        Some(inspection.input.service.uid),
        Some(inspection.input.service.gid),
        Some(0o600),
        4096,
    )?;
    let password = std::str::from_utf8(&raw)
        .map_err(|_| OpsError::message("database secret is not UTF-8"))?
        .trim_end();
    if password.len() != 64 || !password.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(OpsError::message("database secret is malformed"));
    }
    Ok(password.to_string())
}

fn ensure_release_and_assets(
    inspection: &InstallInspection,
    layout: &HostLayout,
    verify_only: bool,
) -> Result<()> {
    let release_root = layout.releases.join(&inspection.release.manifest.commit);
    match fs::symlink_metadata(&release_root) {
        Ok(_) => {
            let _ = crate::install::verify_installed_anchored(
                &release_root,
                &inspection.release.manifest.commit,
                &inspection.release.manifest_sha256,
                0,
                inspection.input.service.gid,
                0o750,
            )?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !verify_only => {
            let result = crate::install::install(
                &inspection.release,
                &release_root,
                crate::install::InstallScope::System {
                    service_uid: inspection.input.service.uid,
                    service_gid: inspection.input.service.gid,
                },
                crate::install::InstallFault::None,
            )?;
            if result != crate::install::InstallResult::Updated {
                return Err(OpsError::message(
                    "fresh first-install release publication did not report an update",
                ));
            }
            let _ = crate::install::verify_installed_anchored(
                &release_root,
                &inspection.release.manifest.commit,
                &inspection.release.manifest_sha256,
                0,
                inspection.input.service.gid,
                0o750,
            )?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(OpsError::message(
                "installed release disappeared after its durable publication phase",
            ));
        }
        Err(error) => {
            return Err(OpsError::context(
                "cannot inspect first-install release destination",
                error,
            ));
        }
    }
    for asset in &inspection.input.assets {
        let configured = inspection
            .config
            .network
            .assets
            .iter()
            .find(|value| value.id == asset.id)
            .ok_or_else(|| OpsError::message("configured runtime asset disappeared"))?;
        ensure_immutable_asset(
            &asset.source,
            Path::new(&configured.path),
            &configured.sha256,
            inspection.input.service.gid,
            verify_only,
        )?;
    }
    for target in inspection.fleet.plugin_targets() {
        let source = release_root.join("jars").join(target.artifact);
        ensure_release_copy(
            &source,
            &target.destination,
            inspection.input.service.gid,
            0,
            inspection.input.service.gid,
            "managed lkjmc plugin",
            verify_only,
        )?;
    }
    let policy =
        crate::eula::canonical_policy_path(&inspection.config, inspection.input.service.gid)?;
    if verify_only {
        crate::eula::verify_materialized(
            &inspection.fleet,
            &policy,
            0,
            inspection.input.service.uid,
            inspection.input.service.gid,
        )?;
    } else {
        let _ = crate::eula::create_policy(&policy, 0, inspection.input.service.gid)?;
        let _ = crate::eula::materialize(
            &inspection.fleet,
            &policy,
            0,
            inspection.input.service.uid,
            inspection.input.service.gid,
        )?;
        crate::eula::verify_materialized(
            &inspection.fleet,
            &policy,
            0,
            inspection.input.service.uid,
            inspection.input.service.gid,
        )?;
    }
    host_deploy::validate_configuration_effects(
        &inspection.config,
        &inspection.fleet,
        inspection.input.service.uid,
        inspection.input.service.gid,
        true,
        &layout.policy,
    )?;
    Ok(())
}

fn ensure_immutable_asset(
    source: &ImmutableFileInput,
    destination: &Path,
    expected_sha256: &str,
    service_gid: u32,
    verify_only: bool,
) -> Result<()> {
    match fs::symlink_metadata(destination) {
        Ok(_) => {
            let metadata = require_regular(
                destination,
                "installed immutable runtime asset",
                Some(0),
                Some(service_gid),
                Some(0o640),
                MAX_RUNTIME_ASSET_BYTES,
            )?;
            if metadata.len() != source.size || sha256_file(destination)? != expected_sha256 {
                return Err(OpsError::message(
                    "installed immutable runtime asset differs from its anchored source",
                ));
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !verify_only => {
            copy_regular(
                &source.path,
                destination,
                0o640,
                0,
                service_gid,
                MAX_RUNTIME_ASSET_BYTES,
            )?;
            ensure_immutable_asset(source, destination, expected_sha256, service_gid, true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(OpsError::message(
            "installed immutable runtime asset disappeared after its durable phase",
        )),
        Err(error) => Err(OpsError::context(
            "cannot inspect installed immutable runtime asset",
            error,
        )),
    }
}

fn ensure_release_copy(
    source: &Path,
    destination: &Path,
    source_gid: u32,
    uid: u32,
    gid: u32,
    label: &str,
    verify_only: bool,
) -> Result<()> {
    let source_metadata = require_regular(
        source,
        "installed release source",
        Some(0),
        Some(source_gid),
        Some(0o640),
        MAX_RUNTIME_ASSET_BYTES,
    )?;
    match fs::symlink_metadata(destination) {
        Ok(_) => {
            let destination_metadata = require_regular(
                destination,
                label,
                Some(uid),
                Some(gid),
                Some(0o640),
                MAX_RUNTIME_ASSET_BYTES,
            )?;
            if destination_metadata.len() != source_metadata.len()
                || sha256_file(destination)? != sha256_file(source)?
            {
                return Err(OpsError::message(format!(
                    "{label} differs from the release"
                )));
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !verify_only => {
            copy_regular(
                source,
                destination,
                0o640,
                uid,
                gid,
                MAX_RUNTIME_ASSET_BYTES,
            )?;
            ensure_release_copy(source, destination, source_gid, uid, gid, label, true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(OpsError::message(
            format!("{label} disappeared after its durable phase"),
        )),
        Err(error) => Err(OpsError::context(&format!("cannot inspect {label}"), error)),
    }
}

fn install_fence(
    inspection: &InstallInspection,
    layout: &HostLayout,
) -> crate::fence::DeploymentFence {
    crate::fence::DeploymentFence {
        schema_version: 2,
        operation: crate::fence::FenceOperation::Install,
        operation_id: inspection.input.operation_id,
        from_commit: None,
        to_commit: inspection.release.manifest.commit.clone(),
        manifest_sha256: inspection.release.manifest_sha256.clone(),
        state_directory: layout.installation_state_directory(inspection.input.operation_id),
        backup: None,
        rollback_snapshot: None,
    }
}

fn ensure_install_fence(
    inspection: &InstallInspection,
    layout: &HostLayout,
    verify_only: bool,
) -> Result<()> {
    let expected = install_fence(inspection, layout);
    match fs::symlink_metadata(&layout.fence) {
        Ok(_) => {
            if crate::fence::read_fence(&layout.fence, 0, 0)? != expected {
                return Err(OpsError::message(
                    "active deployment fence differs from the first-install operation",
                ));
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !verify_only => {
            crate::fence::write_fence(&layout.fence, &expected, 0, 0)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(OpsError::message(
            "first-install deployment fence disappeared after its durable phase",
        )),
        Err(error) => Err(OpsError::context(
            "cannot inspect first-install deployment fence",
            error,
        )),
    }
}

fn ensure_service_publication(
    inspection: &InstallInspection,
    layout: &HostLayout,
    verify_only: bool,
) -> Result<()> {
    let release_root = layout.releases.join(&inspection.release.manifest.commit);
    let dropin_root = layout
        .fence_dropin
        .parent()
        .ok_or_else(|| OpsError::message("systemd fence drop-in has no parent"))?;
    ensure_directory(
        dropin_root,
        0o755,
        0,
        0,
        "systemd fence drop-in directory",
        verify_only,
    )?;
    ensure_release_copy(
        &release_root.join("share/lkjmc-daemon.service"),
        &layout.unit,
        inspection.input.service.gid,
        0,
        0,
        "packaged systemd unit",
        verify_only,
    )?;
    ensure_release_copy(
        &release_root.join("share/lkjmc-deployment-fence.conf"),
        &layout.fence_dropin,
        inspection.input.service.gid,
        0,
        0,
        "packaged systemd fence drop-in",
        verify_only,
    )?;
    ensure_current_pointer(
        &layout.current,
        &inspection.release.manifest.commit,
        verify_only,
    )?;
    if verify_only {
        if !service_enabled()? {
            return Err(OpsError::message(
                "first-install systemd service is not enabled after its durable phase",
            ));
        }
    } else {
        host_deploy::systemctl(&["daemon-reload"], Duration::from_secs(60))?;
        if !service_enabled()? {
            host_deploy::systemctl(&["enable", host_deploy::SERVICE], Duration::from_secs(60))?;
            if !service_enabled()? {
                return Err(OpsError::message(
                    "first-install systemd service did not become enabled",
                ));
            }
        }
    }
    Ok(())
}

fn ensure_current_pointer(path: &Path, commit: &str, verify_only: bool) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_symlink()
                || metadata.uid() != 0
                || fs::read_link(path)
                    .map_err(|error| OpsError::context("cannot read release pointer", error))?
                    != Path::new(commit)
            {
                return Err(OpsError::message(
                    "first-install release pointer differs from the anchored release",
                ));
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && !verify_only => {
            atomic_symlink(Path::new(commit), path, 0)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(OpsError::message(
            "first-install release pointer disappeared after its durable phase",
        )),
        Err(error) => Err(OpsError::context(
            "cannot inspect first-install release pointer",
            error,
        )),
    }
}

fn service_enabled() -> Result<bool> {
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from(host_deploy::SYSTEMCTL),
        arguments: vec!["is-enabled".to_string(), host_deploy::SERVICE.to_string()],
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(30),
        max_output_bytes: 64 * 1024,
    })?;
    match output.status {
        0 => Ok(true),
        1 => Ok(false),
        _ => Err(OpsError::message(
            "cannot determine first-install systemd enablement",
        )),
    }
}

fn activate_service(
    inspection: &InstallInspection,
    layout: &HostLayout,
    verify_only: bool,
) -> Result<()> {
    if !service_is_active_or_activating()? {
        if !verify_only {
            ensure_start_permit(inspection, layout)?;
            host_deploy::systemctl(
                &["start", "--no-block", host_deploy::SERVICE],
                Duration::from_secs(60),
            )?;
        } else {
            return Err(OpsError::message(
                "first-install systemd service is inactive after activation began",
            ));
        }
    }
    if !verify_only {
        invoke_bootstrap_apply(inspection, layout)?;
    }
    verify_live_target(inspection, layout)
}

fn ensure_start_permit(inspection: &InstallInspection, layout: &HostLayout) -> Result<()> {
    match fs::symlink_metadata(&layout.permit) {
        Ok(_) => {
            crate::fence::verify_permit(&layout.permit, &install_fence(inspection, layout), 0, 0)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            crate::fence::write_permit(&layout.permit, &install_fence(inspection, layout), 0, 0)
        }
        Err(error) => Err(OpsError::context(
            "cannot inspect first-install start permit",
            error,
        )),
    }
}

fn service_is_active_or_activating() -> Result<bool> {
    let output = run_bounded(&CommandSpec {
        executable: PathBuf::from(host_deploy::SYSTEMCTL),
        arguments: vec!["is-active".to_string(), host_deploy::SERVICE.to_string()],
        environment: BTreeMap::new(),
        stdin: Vec::new(),
        timeout: Duration::from_secs(30),
        max_output_bytes: 64 * 1024,
    })?;
    let state = std::str::from_utf8(&output.stdout)
        .map_err(|_| OpsError::message("systemd active-state output is not UTF-8"))?
        .trim();
    match (output.status, state) {
        (0, "active") | (3, "activating") => Ok(true),
        (3, "inactive") | (3, "failed") | (3, "deactivating") => Ok(false),
        _ => Err(OpsError::message(
            "systemd active-state output differs from the supported contract",
        )),
    }
}

fn invoke_bootstrap_apply(inspection: &InstallInspection, layout: &HostLayout) -> Result<()> {
    let cli = layout
        .releases
        .join(&inspection.release.manifest.commit)
        .join("bin/lkjmc");
    let deadline = std::time::Instant::now() + Duration::from_secs(1500);
    let mut attempted = false;
    while std::time::Instant::now() < deadline {
        attempted = true;
        let output = run_bounded(&CommandSpec {
            executable: PathBuf::from("/usr/sbin/runuser"),
            arguments: vec![
                "--user".to_string(),
                inspection.input.service.user.clone(),
                "--".to_string(),
                cli.display().to_string(),
                "--socket".to_string(),
                inspection.config.socket_path.clone(),
                "--json".to_string(),
                "bootstrap".to_string(),
                "apply".to_string(),
                "--profile".to_string(),
                "playable".to_string(),
            ],
            environment: BTreeMap::new(),
            stdin: Vec::new(),
            timeout: Duration::from_secs(1500),
            max_output_bytes: 2 * 1024 * 1024,
        })?;
        if output.status == 0 {
            let value: serde_json::Value = serde_json::from_slice(&output.stdout)
                .map_err(|_| OpsError::message("bootstrap apply did not return JSON"))?;
            if bootstrap_response_converged(&value) {
                return Ok(());
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    if attempted {
        Err(OpsError::message(
            "first-install canonical fleet bootstrap did not converge before its deadline",
        ))
    } else {
        Err(OpsError::message(
            "first-install canonical fleet bootstrap was not attempted",
        ))
    }
}

fn bootstrap_response_converged(value: &serde_json::Value) -> bool {
    matches!(
        value.get("result").and_then(serde_json::Value::as_str),
        Some("succeeded" | "no-op")
    )
}

fn verify_live_target(inspection: &InstallInspection, layout: &HostLayout) -> Result<()> {
    let release_root = layout.releases.join(&inspection.release.manifest.commit);
    wait_for_service_running(
        &release_root.join("bin/lkjmc-daemon"),
        inspection.input.service.uid,
    )?;
    crate::bootstrap::after_start_as_user(
        Path::new(CONFIG_ROOT).join("lkjmc.json").as_path(),
        &release_root.join("bin/lkjmc"),
        &inspection.release.manifest.commit,
        Duration::from_secs(120),
        &inspection.input.service.user,
    )?;
    let mut connection = crate::database::connect(&inspection.config, None)?;
    let persisted = crate::database::persisted_inventory(&mut connection.client)?;
    inspection.fleet.compare_persisted(&persisted)?;
    let migrations = crate::database::migration_marker(&mut connection.client)?;
    if migrations.is_empty() {
        return Err(OpsError::message(
            "PostgreSQL migration ledger is empty during first-install acceptance",
        ));
    }
    Ok(())
}

fn wait_for_service_running(expected_executable: &Path, expected_uid: u32) -> Result<()> {
    let deadline = std::time::Instant::now() + Duration::from_secs(120);
    loop {
        let last_failure = match host_deploy::require_service_running_identity(
            expected_executable,
            expected_uid,
        ) {
            Ok(()) => return Ok(()),
            Err(error) => error.to_string(),
        };
        if !service_is_active_or_activating()? {
            return Err(OpsError::message(format!(
                "first-install systemd service did not reach active/running: {last_failure}"
            )));
        }
        if std::time::Instant::now() >= deadline {
            return Err(OpsError::message(format!(
                "first-install systemd service did not reach active/running before its deadline: {last_failure}"
            )));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn finish_acceptance(
    inspection: &InstallInspection,
    layout: &HostLayout,
    journal: &mut InstallJournal,
) -> Result<HostInstallReceipt> {
    verify_accepted_components(inspection, layout, journal)?;
    if fs::symlink_metadata(&layout.permit).is_ok() {
        return Err(OpsError::message(
            "first-install start permit remains after service acceptance",
        ));
    }
    let receipt = receipt_from_inspection(inspection, InstallResult::Accepted)?;
    let mut raw = serde_json::to_vec(&receipt)?;
    raw.push(b'\n');
    let receipt_path = journal.state_directory.join("receipt.json");
    match fs::symlink_metadata(&receipt_path) {
        Ok(_) => {
            let existing = read_regular(
                &receipt_path,
                "first-install final receipt",
                Some(0),
                Some(0),
                Some(0o600),
                MAX_CONTROL_FILE_BYTES,
            )?;
            if existing != raw {
                return Err(OpsError::message(
                    "first-install final receipt differs from the accepted closure",
                ));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            atomic_write(&receipt_path, &raw, 0o600, 0, 0)?;
        }
        Err(error) => {
            return Err(OpsError::context(
                "cannot inspect first-install final receipt",
                error,
            ));
        }
    }
    journal.phase = InstallPhase::Accepted;
    journal.receipt_sha256 = Some(sha256_bytes(&raw));
    journal.fence_cleared = false;
    write_install_journal(&journal.state_directory.join("journal.json"), journal)?;
    finalize_accepted_fence(inspection, layout, journal)?;
    Ok(receipt)
}

fn finalize_accepted_fence(
    inspection: &InstallInspection,
    layout: &HostLayout,
    journal: &mut InstallJournal,
) -> Result<()> {
    if journal.phase != InstallPhase::Accepted {
        return Err(OpsError::message(
            "only an accepted first-install journal may finalize the deployment fence",
        ));
    }
    if journal.fence_cleared {
        return Ok(());
    }
    verify_accepted_components(inspection, layout, journal)?;
    if fs::symlink_metadata(&layout.permit).is_ok() {
        return Err(OpsError::message(
            "first-install start permit remains while finalizing acceptance",
        ));
    }
    match fs::symlink_metadata(&layout.fence) {
        Ok(_) => {
            if crate::fence::read_fence(&layout.fence, 0, 0)? != install_fence(inspection, layout) {
                return Err(OpsError::message(
                    "deployment fence differs while finalizing accepted first install",
                ));
            }
            crate::fence::remove_fence(&layout.fence, 0, 0)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(OpsError::context(
                "cannot inspect deployment fence while finalizing accepted first install",
                error,
            ));
        }
    }
    journal.fence_cleared = true;
    write_install_journal(&journal.state_directory.join("journal.json"), journal)
}

fn verify_accepted_target(
    inspection: &InstallInspection,
    layout: &HostLayout,
    journal: &InstallJournal,
) -> Result<()> {
    journal.matches(
        inspection,
        &layout.installation_state_directory(inspection.input.operation_id),
    )?;
    if journal.phase != InstallPhase::Accepted {
        return Err(OpsError::message(
            "first-install target is not accepted and cannot be classified as no-op",
        ));
    }
    if !journal.fence_cleared {
        return Err(OpsError::message(
            "accepted first-install journal has not completed deployment-fence finalization",
        ));
    }
    if fs::symlink_metadata(&layout.fence).is_ok() || fs::symlink_metadata(&layout.permit).is_ok() {
        return Err(OpsError::message(
            "accepted first-install target is fenced or has an orphan start permit",
        ));
    }
    ensure_identity_and_roots(inspection, true)?;
    ensure_secrets(inspection, true)?;
    ensure_database(inspection, true)?;
    ensure_release_and_assets(inspection, layout, true)?;
    ensure_service_publication(inspection, layout, true)?;
    verify_live_target(inspection, layout)?;
    let receipt = read_final_receipt(inspection, journal, InstallResult::Accepted)?;
    if receipt.result != InstallResult::Accepted {
        return Err(OpsError::message(
            "durable first-install receipt has an unexpected result",
        ));
    }
    Ok(())
}

fn verify_accepted_components(
    inspection: &InstallInspection,
    layout: &HostLayout,
    journal: &InstallJournal,
) -> Result<()> {
    ensure_identity_and_roots(inspection, true)?;
    ensure_secrets(inspection, true)?;
    ensure_database(inspection, true)?;
    ensure_release_and_assets(inspection, layout, true)?;
    ensure_install_fence(inspection, layout, true)?;
    ensure_service_publication(inspection, layout, true)?;
    verify_live_target(inspection, layout)?;
    if journal.state_directory != layout.installation_state_directory(inspection.input.operation_id)
    {
        return Err(OpsError::message(
            "first-install journal state directory differs during acceptance",
        ));
    }
    Ok(())
}

fn receipt_from_inspection(
    inspection: &InstallInspection,
    result: InstallResult,
) -> Result<HostInstallReceipt> {
    Ok(HostInstallReceipt {
        schema_version: 1,
        result,
        operation_id: inspection.input.operation_id,
        release_commit: inspection.release.manifest.commit.clone(),
        release_manifest_sha256: inspection.release.manifest_sha256.clone(),
        install_input_sha256: inspection.input_sha256.clone(),
        configuration_sha256: inspection.input.configuration.sha256.clone(),
        runtime_asset_closure_sha256: inspection.asset_closure_sha256.clone(),
        fleet_revision: inspection.fleet.revision,
        instance_ids: inspection
            .fleet
            .instances()
            .map(|instance| instance.id.as_str().to_string())
            .collect(),
        velocity_instance_id: inspection.fleet.velocity_entry()?.id.as_str().to_string(),
    })
}

fn read_final_receipt(
    inspection: &InstallInspection,
    journal: &InstallJournal,
    result: InstallResult,
) -> Result<HostInstallReceipt> {
    let raw = read_regular(
        &journal.state_directory.join("receipt.json"),
        "first-install final receipt",
        Some(0),
        Some(0),
        Some(0o600),
        MAX_CONTROL_FILE_BYTES,
    )?;
    let receipt_sha256 = sha256_bytes(&raw);
    if journal.receipt_sha256.as_deref() != Some(receipt_sha256.as_str()) {
        return Err(OpsError::message(
            "first-install final receipt digest differs from the durable journal",
        ));
    }
    let mut receipt: HostInstallReceipt = serde_json::from_slice(&raw)
        .map_err(|error| OpsError::context("invalid first-install final receipt", error))?;
    let expected = receipt_from_inspection(inspection, InstallResult::Accepted)?;
    if receipt != expected {
        return Err(OpsError::message(
            "first-install final receipt differs from the accepted input closure",
        ));
    }
    receipt.result = result;
    Ok(receipt)
}

pub(crate) fn inspect(request: &HostInstallRequest) -> Result<InstallInspection> {
    request.validate()?;
    validate_ancestry(&request.input_path, Path::new("/"), 0)?;
    let raw = read_regular(
        &request.input_path,
        "first-install input",
        Some(0),
        Some(0),
        Some(0o600),
        INPUT_MAX_BYTES,
    )?;
    let input_sha256 = sha256_bytes(&raw);
    if input_sha256 != request.input_sha256 {
        return Err(OpsError::message(
            "first-install input differs from the anchored SHA-256",
        ));
    }
    let input: HostInstallInput = serde_json::from_slice(&raw)
        .map_err(|error| OpsError::context("invalid first-install input", error))?;
    validate_input_shape(&input)?;
    let config_bytes = read_immutable_source(
        &input.configuration,
        "first-install canonical configuration",
        INPUT_MAX_BYTES,
    )?;
    let config = std::str::from_utf8(&config_bytes)
        .map_err(|_| OpsError::message("first-install canonical configuration is not UTF-8"))
        .and_then(|text| {
            LkjmcConfig::from_json_str(text)
                .map_err(|error| OpsError::context("invalid first-install configuration", error))
        })?;
    let fleet = FleetSnapshot::from_config(&config)?;
    validate_config_contract(&input, &config, &fleet)?;
    let release =
        VerifiedRelease::load_anchored(&input.release.root, &input.release.manifest_sha256)?;
    crate::install::validate_system_release_source(&release)?;
    if release.manifest.commit != input.release.commit {
        return Err(OpsError::message(
            "anchored release commit differs from first-install input",
        ));
    }
    let asset_closure_sha256 = validate_asset_sources(&input, &config)?;
    Ok(InstallInspection {
        input,
        input_sha256,
        release,
        config_bytes,
        config,
        fleet,
        asset_closure_sha256,
    })
}

fn validate_input_shape(input: &HostInstallInput) -> Result<()> {
    if input.schema_version != 1 {
        return Err(OpsError::message("unsupported first-install input schema"));
    }
    if input.operation_id.is_nil() {
        return Err(OpsError::message(
            "first-install operation UUID must not be the nil UUID",
        ));
    }
    require_hex(&input.release.commit, 40, "first-install release commit")?;
    require_hex(
        &input.release.manifest_sha256,
        64,
        "first-install release manifest SHA-256",
    )?;
    require_absolute_safe(&input.release.root, "first-install release root")?;
    validate_immutable_file(
        &input.configuration,
        "first-install canonical configuration",
        INPUT_MAX_BYTES,
    )?;
    if input.assets.is_empty() || input.assets.len() > 256 {
        return Err(OpsError::message(
            "first-install runtime asset inventory must contain 1..=256 entries",
        ));
    }
    let mut asset_ids = BTreeSet::new();
    for asset in &input.assets {
        validate_identifier(&asset.id, "first-install runtime asset ID")?;
        if !asset_ids.insert(asset.id.as_str()) {
            return Err(OpsError::message(
                "first-install runtime asset IDs must be unique",
            ));
        }
        if asset.kind != AssetKind::Server {
            return Err(OpsError::message(
                "first-install currently supports only immutable server runtime assets",
            ));
        }
        validate_bounded_text(&asset.version, "first-install runtime asset version")?;
        validate_bounded_text(
            &asset.source_identity,
            "first-install runtime asset source identity",
        )?;
        validate_immutable_file(
            &asset.source,
            "first-install runtime asset source",
            MAX_RUNTIME_ASSET_BYTES,
        )?;
    }
    validate_roots(&input.roots)?;
    validate_service_contract(&input.service)?;
    validate_postgres_contract(&input.postgres)?;
    validate_capacity_contract(&input.capacity)
}

fn validate_roots(roots: &InstallRoots) -> Result<()> {
    let expected = [
        (
            &roots.install_root,
            INSTALL_ROOT,
            "first-install install root",
        ),
        (
            &roots.config_root,
            CONFIG_ROOT,
            "first-install configuration root",
        ),
        (&roots.data_root, DATA_ROOT, "first-install data root"),
        (&roots.log_root, LOG_ROOT, "first-install log root"),
        (&roots.backup_root, BACKUP_ROOT, "first-install backup root"),
        (
            &roots.runtime_asset_root,
            RUNTIME_ASSET_ROOT,
            "first-install runtime asset root",
        ),
    ];
    for (path, expected_path, label) in expected {
        require_absolute_safe(path, label)?;
        if path != Path::new(expected_path) {
            return Err(OpsError::message(format!(
                "{label} differs from the packaged systemd contract"
            )));
        }
    }
    Ok(())
}

fn validate_service_contract(service: &ServiceContract) -> Result<()> {
    if service.user != SERVICE_USER
        || service.group != SERVICE_GROUP
        || service.home != Path::new(SERVICE_HOME)
        || service.shell != Path::new(SERVICE_SHELL)
    {
        return Err(OpsError::message(
            "first-install service identity differs from the packaged systemd contract",
        ));
    }
    if service.uid == 0 || service.gid == 0 || service.uid > 60000 || service.gid > 60000 {
        return Err(OpsError::message(
            "first-install service numeric identity is outside the supported unprivileged range",
        ));
    }
    Ok(())
}

fn validate_postgres_contract(postgres: &PostgresContract) -> Result<()> {
    if postgres.administrative_user != POSTGRES_ADMIN
        || postgres.socket_directory != Path::new(POSTGRES_SOCKET)
    {
        return Err(OpsError::message(
            "first-install PostgreSQL administrative boundary differs from the supported local contract",
        ));
    }
    validate_postgres_identifier(&postgres.role, "first-install PostgreSQL role")?;
    validate_postgres_identifier(&postgres.database, "first-install PostgreSQL database")
}

fn validate_capacity_contract(capacity: &CapacityContract) -> Result<()> {
    if !(128..=16 * 1024 * 1024).contains(&capacity.minimum_free_mib)
        || !(512..=16 * 1024 * 1024).contains(&capacity.minimum_memory_mib)
        || !(64..=1_000_000).contains(&capacity.minimum_processes)
        || !(1_024..=10_000_000).contains(&capacity.minimum_open_files)
    {
        return Err(OpsError::message(
            "first-install capacity contract is outside supported bounds",
        ));
    }
    Ok(())
}

fn validate_config_contract(
    input: &HostInstallInput,
    config: &LkjmcConfig,
    fleet: &FleetSnapshot,
) -> Result<()> {
    let roots = &input.roots;
    let expected = [
        (
            &config.install_root,
            roots.install_root.as_path(),
            "configured install root",
        ),
        (
            &config.config_root,
            roots.config_root.as_path(),
            "configured configuration root",
        ),
        (
            &config.data_root,
            roots.data_root.as_path(),
            "configured data root",
        ),
        (
            &config.log_root,
            roots.log_root.as_path(),
            "configured log root",
        ),
        (
            &config.socket_path,
            Path::new(SOCKET_PATH),
            "configured daemon socket",
        ),
        (
            &config.jars.root,
            Path::new(JAR_ROOT),
            "configured immutable jar root",
        ),
        (
            &config.assets.root,
            roots.runtime_asset_root.as_path(),
            "configured runtime asset root",
        ),
        (
            &config.database.secret_file,
            Path::new(DATABASE_SECRET),
            "configured database secret file",
        ),
        (
            &config.network.forwarding.secret_file,
            Path::new(FORWARDING_SECRET),
            "configured forwarding secret file",
        ),
    ];
    for (observed, expected, label) in expected {
        if Path::new(observed) != expected {
            return Err(OpsError::message(format!(
                "{label} differs from the first-install system contract"
            )));
        }
    }
    if config.runtime.adapter != RuntimeAdapter::LocalProcess
        || !config.network.capabilities.mounted_config
        || !config.network.capabilities.mounted_secrets
        || !config.network.capabilities.mounted_assets
        || !config.plugins.lkjmc.enabled
    {
        return Err(OpsError::message(
            "first-install requires the packaged local-process and lkjmc-plugin contract",
        ));
    }
    if !config.daemon_http.enabled || config.daemon_http.token_file != HTTP_TOKEN {
        return Err(OpsError::message(
            "first-install requires the packaged private daemon HTTP credential path",
        ));
    }
    if config.database.host != "127.0.0.1"
        || config.database.user != input.postgres.role
        || config.database.database != input.postgres.database
    {
        return Err(OpsError::message(
            "configured PostgreSQL target differs from the first-install contract",
        ));
    }
    if fleet.velocity_entry()?.id.as_str().is_empty()
        || fleet.backends().next().is_none()
        || fleet.instances().any(|instance| {
            instance.desired_state.requires_service() && instance.asset_ids.is_empty()
        })
    {
        return Err(OpsError::message(
            "first-install fleet lacks a supported Velocity entrypoint, backend, or runtime asset",
        ));
    }
    for asset in &config.network.assets {
        let path = Path::new(&asset.path);
        if !asset.required
            || asset.kind != AssetKind::Server
            || path.parent() != Some(roots.runtime_asset_root.as_path())
        {
            return Err(OpsError::message(
                "first-install runtime assets must be required server bytes directly under the managed asset root",
            ));
        }
    }
    Ok(())
}

fn validate_asset_sources(input: &HostInstallInput, config: &LkjmcConfig) -> Result<String> {
    let configured = config
        .network
        .assets
        .iter()
        .map(|asset| (asset.id.as_str(), asset))
        .collect::<BTreeMap<_, _>>();
    let supplied = input
        .assets
        .iter()
        .map(|asset| (asset.id.as_str(), asset))
        .collect::<BTreeMap<_, _>>();
    if configured.keys().collect::<BTreeSet<_>>() != supplied.keys().collect::<BTreeSet<_>>() {
        return Err(OpsError::message(
            "first-install runtime asset closure differs from canonical configuration",
        ));
    }
    for (id, configured_asset) in configured {
        let supplied_asset = supplied
            .get(id)
            .ok_or_else(|| OpsError::message("first-install runtime asset disappeared"))?;
        if configured_asset.kind != supplied_asset.kind
            || configured_asset.sha256 != supplied_asset.source.sha256
        {
            return Err(OpsError::message(format!(
                "first-install runtime asset identity differs: {id}"
            )));
        }
        let bytes = read_immutable_source(
            &supplied_asset.source,
            "first-install runtime asset source",
            MAX_RUNTIME_ASSET_BYTES,
        )?;
        if sha256_bytes(&bytes) != configured_asset.sha256 {
            return Err(OpsError::message(format!(
                "first-install runtime asset digest differs after anchored read: {id}"
            )));
        }
    }
    let canonical = input
        .assets
        .iter()
        .map(|asset| (asset.id.clone(), asset))
        .collect::<BTreeMap<_, _>>();
    serde_json::to_vec(&canonical)
        .map(|bytes| sha256_bytes(&bytes))
        .map_err(|error| OpsError::context("cannot serialize runtime asset closure", error))
}

fn read_immutable_source(
    source: &ImmutableFileInput,
    label: &str,
    maximum: u64,
) -> Result<Vec<u8>> {
    validate_immutable_file(source, label, maximum)?;
    validate_ancestry(&source.path, Path::new("/"), 0)?;
    let bytes = read_regular(&source.path, label, Some(0), Some(0), Some(0o600), maximum)?;
    if bytes.len() as u64 != source.size || sha256_bytes(&bytes) != source.sha256 {
        return Err(OpsError::message(format!(
            "{label} differs from its anchored identity"
        )));
    }
    Ok(bytes)
}

fn validate_immutable_file(source: &ImmutableFileInput, label: &str, maximum: u64) -> Result<()> {
    require_absolute_safe(&source.path, label)?;
    require_hex(&source.sha256, 64, &format!("{label} SHA-256"))?;
    if source.size == 0 || source.size > maximum {
        return Err(OpsError::message(format!(
            "{label} size is outside its bound"
        )));
    }
    Ok(())
}

fn validate_identifier(value: &str, label: &str) -> Result<()> {
    lkjmc_core::id::InstanceId::parse(value.to_string())
        .map(|_| ())
        .map_err(|error| OpsError::context(label, error))
}

fn validate_postgres_identifier(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 63
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(OpsError::message(format!(
            "{label} is not a safe PostgreSQL identifier"
        )));
    }
    Ok(())
}

fn validate_bounded_text(value: &str, label: &str) -> Result<()> {
    if value.is_empty() || value.len() > 512 || value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(OpsError::message(format!(
            "{label} is empty, too long, or unsafe"
        )));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_install_contract_rejects_unknown_or_unsafe_identity() -> Result<()> {
        let mut input = fixture();
        validate_input_shape(&input)?;
        input.service.uid = 0;
        assert!(validate_input_shape(&input).is_err());
        input = fixture();
        input.postgres.role = "bad-role".to_string();
        assert!(validate_input_shape(&input).is_err());
        input = fixture();
        input.assets[0].version = "\n".to_string();
        assert!(validate_input_shape(&input).is_err());
        input = fixture();
        input.operation_id = Uuid::nil();
        assert!(validate_input_shape(&input).is_err());
        Ok(())
    }

    #[test]
    fn first_install_asset_closure_identity_is_order_independent() -> Result<()> {
        let mut input = fixture();
        input.assets.push(RuntimeAssetInput {
            id: "quartz-server".to_string(),
            kind: AssetKind::Server,
            version: "1.21.8".to_string(),
            source_identity: "paper-build-1".to_string(),
            source: ImmutableFileInput {
                path: PathBuf::from("/root/input/quartz.jar"),
                sha256: "b".repeat(64),
                size: 1024,
            },
        });
        let first = serde_json::to_vec(
            &input
                .assets
                .iter()
                .map(|asset| (asset.id.clone(), asset))
                .collect::<BTreeMap<_, _>>(),
        )?;
        input.assets.reverse();
        let second = serde_json::to_vec(
            &input
                .assets
                .iter()
                .map(|asset| (asset.id.clone(), asset))
                .collect::<BTreeMap<_, _>>(),
        )?;
        assert_eq!(sha256_bytes(&first), sha256_bytes(&second));
        Ok(())
    }

    #[test]
    fn accepted_journal_requires_a_receipt_and_is_the_only_fence_clear_state() -> Result<()> {
        let mut journal = journal_fixture();
        journal.phase = InstallPhase::Accepted;
        assert!(journal.validate().is_err());
        journal.receipt_sha256 = Some("f".repeat(64));
        journal.fence_cleared = true;
        journal.validate()?;
        journal.phase = InstallPhase::Activating;
        assert!(journal.validate().is_err());
        Ok(())
    }

    #[test]
    fn bootstrap_response_uses_the_cli_body_contract_not_the_transport_envelope() {
        assert!(bootstrap_response_converged(
            &serde_json::json!({"result":"succeeded"})
        ));
        assert!(bootstrap_response_converged(
            &serde_json::json!({"result":"no-op"})
        ));
        assert!(!bootstrap_response_converged(
            &serde_json::json!({"ok":true})
        ));
        assert!(!bootstrap_response_converged(
            &serde_json::json!({"result":"failed"})
        ));
    }

    #[test]
    fn plugin_heartbeat_bindings_require_the_exact_instance_scope() -> Result<()> {
        let target = crate::fleet::CredentialTarget {
            instance_id: lkjmc_core::id::InstanceId::parse("alpha-world".to_string())
                .map_err(|error| OpsError::context("invalid test instance ID", error))?,
            surface: "paper",
            path: PathBuf::from("/var/lib/lkjmc/private/plugin-credentials/alpha-world.secret"),
        };
        let mut record = lkjmc_store::daemon_token::DaemonTokenRecord {
            credential_id: Uuid::nil(),
            surface: "paper".to_string(),
            principal_kind: "instance".to_string(),
            principal_id: "alpha-world".to_string(),
            scopes: vec![PLUGIN_HEARTBEAT_SCOPE.to_string()],
            expires_at_micros: i64::MAX,
        };
        assert!(plugin_heartbeat_record_matches(&record, &target));
        record.scopes.push("lkjmc.sync.read".to_string());
        assert!(!plugin_heartbeat_record_matches(&record, &target));
        record.scopes = vec![PLUGIN_HEARTBEAT_SCOPE.to_string()];
        record.surface = "velocity".to_string();
        assert!(!plugin_heartbeat_record_matches(&record, &target));
        Ok(())
    }

    #[test]
    fn plugin_heartbeat_secret_matches_the_generated_file_contract() -> Result<()> {
        let secret = "a".repeat(64);
        let generated = format!("{secret}\n");
        assert_eq!(parse_plugin_heartbeat_secret(generated.as_bytes())?, secret);
        assert!(parse_plugin_heartbeat_secret(b"not-a-secret\n").is_err());
        assert!(
            parse_plugin_heartbeat_secret(format!(" {}\n", "a".repeat(63)).as_bytes()).is_err()
        );
        Ok(())
    }

    #[test]
    fn shared_syslog_log_parent_is_the_only_writable_parent_exception() -> Result<()> {
        assert_eq!(parse_group_gid("syslog:x:102:", "syslog")?, 102);
        assert!(is_standard_syslog_log_parent(
            Path::new("/var/log"),
            0,
            102,
            0o775,
            Some(102),
        ));
        for (path, uid, gid, mode) in [
            (Path::new("/var/log"), 1000, 102, 0o775),
            (Path::new("/var/log"), 0, 1000, 0o775),
            (Path::new("/var/log"), 0, 102, 0o777),
            (Path::new("/var/lib"), 0, 102, 0o775),
        ] {
            assert!(!is_standard_syslog_log_parent(
                path,
                uid,
                gid,
                mode,
                Some(102),
            ));
        }
        assert!(parse_group_gid("syslog:x:not-a-gid:", "syslog").is_err());
        assert!(parse_group_gid("other:x:102:", "syslog").is_err());
        Ok(())
    }

    fn journal_fixture() -> InstallJournal {
        InstallJournal {
            schema_version: 2,
            operation_id: Uuid::from_u128(1),
            input_sha256: "a".repeat(64),
            release_commit: "b".repeat(40),
            release_manifest_sha256: "c".repeat(64),
            configuration_sha256: "d".repeat(64),
            asset_closure_sha256: "e".repeat(64),
            state_directory: PathBuf::from(
                "/var/lib/private/lkjmc-installations/00000000-0000-0000-0000-000000000001",
            ),
            phase: InstallPhase::Prepared,
            first_failure: None,
            receipt_sha256: None,
            fence_cleared: false,
        }
    }

    fn fixture() -> HostInstallInput {
        HostInstallInput {
            schema_version: 1,
            operation_id: Uuid::from_u128(1),
            release: ReleaseInput {
                root: PathBuf::from("/root/release"),
                commit: "a".repeat(40),
                manifest_sha256: "b".repeat(64),
            },
            configuration: ImmutableFileInput {
                path: PathBuf::from("/root/input/lkjmc.json"),
                sha256: "c".repeat(64),
                size: 1024,
            },
            assets: vec![RuntimeAssetInput {
                id: "edge-server".to_string(),
                kind: AssetKind::Server,
                version: "3.4.0".to_string(),
                source_identity: "velocity-build-1".to_string(),
                source: ImmutableFileInput {
                    path: PathBuf::from("/root/input/edge.jar"),
                    sha256: "d".repeat(64),
                    size: 1024,
                },
            }],
            roots: InstallRoots {
                install_root: PathBuf::from(INSTALL_ROOT),
                config_root: PathBuf::from(CONFIG_ROOT),
                data_root: PathBuf::from(DATA_ROOT),
                log_root: PathBuf::from(LOG_ROOT),
                backup_root: PathBuf::from(BACKUP_ROOT),
                runtime_asset_root: PathBuf::from(RUNTIME_ASSET_ROOT),
            },
            service: ServiceContract {
                user: SERVICE_USER.to_string(),
                group: SERVICE_GROUP.to_string(),
                uid: 999,
                gid: 999,
                home: PathBuf::from(SERVICE_HOME),
                shell: PathBuf::from(SERVICE_SHELL),
            },
            postgres: PostgresContract {
                administrative_user: POSTGRES_ADMIN.to_string(),
                socket_directory: PathBuf::from(POSTGRES_SOCKET),
                role: "lkjmc".to_string(),
                database: "lkjmc".to_string(),
            },
            capacity: CapacityContract {
                minimum_free_mib: 1024,
                minimum_memory_mib: 2048,
                minimum_processes: 256,
                minimum_open_files: 65535,
            },
        }
    }
}
