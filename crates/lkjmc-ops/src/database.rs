use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use lkjmc_core::config::LkjmcConfig;
use postgres::{Client, Config, NoTls};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{OpsError, Result};
use crate::fleet::PersistedInstance;
use crate::journal::{BackupClosure, MigrationIdentity};
use crate::manifest::{sha256_bytes, sha256_file};
use crate::process::{require_success, run_bounded, CommandSpec};
use crate::secure_fs::{
    atomic_write, create_directory, effective_gid, effective_uid, read_regular,
    require_absolute_safe, require_directory, require_regular, sync_directory,
    MAX_CONTROL_FILE_BYTES,
};

const PG_DUMP: &str = "/usr/bin/pg_dump";
const PG_RESTORE: &str = "/usr/bin/pg_restore";
const MAX_DUMP_BYTES: u64 = 64 * 1024 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupMetadata {
    pub schema_version: u32,
    pub source_commit: String,
    pub server_version: String,
    pub created_at_unix_seconds: u64,
    pub migration_marker: Vec<MigrationIdentity>,
    pub migration_identity: String,
    pub schema_identity: String,
    pub dump_sha256: String,
    pub dump_size: u64,
    pub manifest_sha256: String,
    pub manifest_size: u64,
}

pub struct DatabaseConnection {
    pub client: Client,
    password: String,
}

pub fn connect(
    config: &LkjmcConfig,
    database_override: Option<&str>,
) -> Result<DatabaseConnection> {
    let secret_path = Path::new(&config.database.secret_file);
    require_absolute_safe(secret_path, "database secret path")?;
    let secret = read_regular(secret_path, "database secret", None, None, None, 4096)?;
    let password = parse_secret(&secret)?;
    let database = database_override.unwrap_or(&config.database.database);
    require_database_name(database)?;
    let mut connection = Config::new();
    connection
        .host(&config.database.host)
        .port(config.database.port)
        .dbname(database)
        .user(&config.database.user)
        .password(&password)
        .connect_timeout(Duration::from_secs(10));
    let mut client = connection
        .connect(NoTls)
        .map_err(|_| OpsError::message("PostgreSQL connection failed"))?;
    client
        .batch_execute("set statement_timeout = '30s'; set lock_timeout = '10s'")
        .map_err(|_| OpsError::message("PostgreSQL session bounds could not be established"))?;
    Ok(DatabaseConnection { client, password })
}

pub fn migration_marker(client: &mut Client) -> Result<Vec<MigrationIdentity>> {
    let rows = client
        .query(
            "select version, name, coalesce(checksum, '') from schema_migrations order by version",
            &[],
        )
        .map_err(|_| OpsError::message("cannot read PostgreSQL migration ledger"))?;
    rows.into_iter()
        .map(|row| {
            let version: i32 = row.get(0);
            let version = u32::try_from(version).map_err(|_| {
                OpsError::message("migration version is outside the supported range")
            })?;
            let value = MigrationIdentity {
                version,
                name: row.get(1),
                checksum: row.get(2),
            };
            validate_migration(&value)?;
            Ok(value)
        })
        .collect()
}

pub fn persisted_inventory(client: &mut Client) -> Result<Vec<PersistedInstance>> {
    let rows = client
        .query(
            "select id, kind, desired_state from instances order by id",
            &[],
        )
        .map_err(|_| OpsError::message("cannot read persisted instance inventory"))?;
    Ok(rows
        .into_iter()
        .map(|row| PersistedInstance {
            id: row.get(0),
            kind: row.get(1),
            desired_state: row.get(2),
        })
        .collect())
}

pub fn create_backup(
    config: &LkjmcConfig,
    destination: &Path,
    source_commit: &str,
) -> Result<BackupClosure> {
    require_hex(source_commit, 40, "backup source commit")?;
    require_absolute_safe(destination, "backup destination")?;
    if destination.exists() || fs::symlink_metadata(destination).is_ok() {
        return Err(OpsError::message("backup destination already exists"));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| OpsError::message("backup destination has no parent"))?;
    require_directory(parent, "backup parent", None, None, None)?;
    let uid = effective_uid();
    let gid = effective_gid();
    let stage = sibling(destination, ".lkjmc-backup")?;
    create_directory(&stage, 0o700, uid, gid)?;
    let result = (|| {
        let mut database = connect(config, None)?;
        let before = migration_marker(&mut database.client)?;
        let schema_before = schema_identity(&mut database.client)?;
        let server_version: String = database
            .client
            .query_one("show server_version", &[])
            .map_err(|_| OpsError::message("cannot read PostgreSQL server version"))?
            .get(0);
        let dump = stage.join("database.dump");
        let output = run_bounded(&postgres_command(
            Path::new(PG_DUMP),
            config,
            &database.password,
            vec![
                "--format=custom".to_string(),
                "--no-owner".to_string(),
                "--no-privileges".to_string(),
                "--file".to_string(),
                path_text(&dump)?,
                config.database.database.clone(),
            ],
            Duration::from_secs(900),
        ))?;
        require_success(output, "PostgreSQL backup")?;
        fs::set_permissions(&dump, fs::Permissions::from_mode(0o600))
            .map_err(|error| OpsError::context("cannot set backup dump mode", error))?;
        std::os::unix::fs::chown(&dump, Some(uid), Some(gid))
            .map_err(|error| OpsError::context("cannot set backup dump ownership", error))?;
        require_regular(
            &dump,
            "PostgreSQL backup dump",
            Some(uid),
            Some(gid),
            Some(0o600),
            MAX_DUMP_BYTES,
        )?;
        let manifest_output = require_success(
            run_bounded(&CommandSpec {
                executable: PathBuf::from(PG_RESTORE),
                arguments: vec!["--list".to_string(), path_text(&dump)?],
                environment: BTreeMap::new(),
                stdin: Vec::new(),
                timeout: Duration::from_secs(120),
                max_output_bytes: MAX_MANIFEST_BYTES as usize,
            })?,
            "PostgreSQL backup structural inspection",
        )?;
        if manifest_output.stdout.is_empty() {
            return Err(OpsError::message(
                "PostgreSQL backup structural manifest is empty",
            ));
        }
        let manifest = stage.join("database.manifest");
        atomic_write(&manifest, &manifest_output.stdout, 0o600, uid, gid)?;
        let after = migration_marker(&mut database.client)?;
        if before != after || schema_before != schema_identity(&mut database.client)? {
            return Err(OpsError::message(
                "PostgreSQL schema or migration ledger changed during backup",
            ));
        }
        let dump_metadata = fs::metadata(&dump)
            .map_err(|error| OpsError::context("cannot inspect backup dump", error))?;
        let manifest_metadata = fs::metadata(&manifest)
            .map_err(|error| OpsError::context("cannot inspect backup manifest", error))?;
        let migration_identity = migration_identity(&before)?;
        let metadata = BackupMetadata {
            schema_version: 1,
            source_commit: source_commit.to_string(),
            server_version,
            created_at_unix_seconds: unix_seconds()?,
            migration_marker: before,
            migration_identity,
            schema_identity: schema_before,
            dump_sha256: sha256_file(&dump)?,
            dump_size: dump_metadata.len(),
            manifest_sha256: sha256_file(&manifest)?,
            manifest_size: manifest_metadata.len(),
        };
        let metadata_path = stage.join("metadata.json");
        let mut metadata_raw = serde_json::to_vec(&metadata)?;
        metadata_raw.push(b'\n');
        atomic_write(&metadata_path, &metadata_raw, 0o600, uid, gid)?;
        let checksums = stage.join("checksums.sha256");
        let checksums_raw = format!(
            "{}  database.dump\n{}  database.manifest\n{}  metadata.json\n",
            metadata.dump_sha256,
            metadata.manifest_sha256,
            sha256_bytes(&metadata_raw)
        );
        atomic_write(&checksums, checksums_raw.as_bytes(), 0o600, uid, gid)?;
        sync_directory(&stage)?;
        drop(database);
        fs::rename(&stage, destination).map_err(|error| {
            OpsError::context("cannot atomically publish backup closure", error)
        })?;
        sync_directory(parent)?;
        verify_backup(config, destination, Some(source_commit), 3600)
    })();
    if result.is_err() && stage.exists() {
        remove_owned_stage(&stage, destination)?;
    }
    result
}

pub fn verify_backup(
    _config: &LkjmcConfig,
    root: &Path,
    expected_source_commit: Option<&str>,
    max_age_seconds: u64,
) -> Result<BackupClosure> {
    require_absolute_safe(root, "backup root")?;
    if !(60..=604_800).contains(&max_age_seconds) {
        return Err(OpsError::message(
            "backup maximum age must be between 60 and 604800 seconds",
        ));
    }
    let uid = effective_uid();
    let gid = effective_gid();
    require_directory(root, "backup closure", Some(uid), Some(gid), Some(0o700))?;
    let expected_names = [
        "checksums.sha256",
        "database.dump",
        "database.manifest",
        "metadata.json",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect::<BTreeSet<_>>();
    let names = fs::read_dir(root)
        .map_err(|error| OpsError::context("cannot enumerate backup closure", error))?
        .map(|entry| {
            entry
                .map_err(|error| OpsError::context("cannot read backup entry", error))?
                .file_name()
                .into_string()
                .map_err(|_| OpsError::message("backup filename is not UTF-8"))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    if names != expected_names {
        return Err(OpsError::message("backup closure file set differs"));
    }
    let dump = root.join("database.dump");
    let manifest = root.join("database.manifest");
    let metadata_path = root.join("metadata.json");
    let checksums = root.join("checksums.sha256");
    let _ = require_regular(
        &dump,
        "backup dump",
        Some(uid),
        Some(gid),
        Some(0o600),
        MAX_DUMP_BYTES,
    )?;
    let manifest_raw = read_regular(
        &manifest,
        "backup manifest",
        Some(uid),
        Some(gid),
        Some(0o600),
        MAX_MANIFEST_BYTES,
    )?;
    let metadata_raw = read_regular(
        &metadata_path,
        "backup metadata",
        Some(uid),
        Some(gid),
        Some(0o600),
        MAX_CONTROL_FILE_BYTES,
    )?;
    let checksums_raw = read_regular(
        &checksums,
        "backup checksums",
        Some(uid),
        Some(gid),
        Some(0o600),
        4096,
    )?;
    let metadata: BackupMetadata = serde_json::from_slice(&metadata_raw)
        .map_err(|error| OpsError::context("invalid backup metadata", error))?;
    validate_metadata(&metadata, expected_source_commit, max_age_seconds)?;
    if fs::metadata(&dump)
        .map_err(|error| OpsError::context("cannot inspect backup dump", error))?
        .len()
        != metadata.dump_size
        || sha256_file(&dump)? != metadata.dump_sha256
        || manifest_raw.len() as u64 != metadata.manifest_size
        || sha256_bytes(&manifest_raw) != metadata.manifest_sha256
    {
        return Err(OpsError::message("backup bytes differ from metadata"));
    }
    let expected_checksums = format!(
        "{}  database.dump\n{}  database.manifest\n{}  metadata.json\n",
        metadata.dump_sha256,
        metadata.manifest_sha256,
        sha256_bytes(&metadata_raw)
    );
    if checksums_raw != expected_checksums.as_bytes() {
        return Err(OpsError::message("backup checksum closure differs"));
    }
    let observed_manifest = require_success(
        run_bounded(&CommandSpec {
            executable: PathBuf::from(PG_RESTORE),
            arguments: vec!["--list".to_string(), path_text(&dump)?],
            environment: BTreeMap::new(),
            stdin: Vec::new(),
            timeout: Duration::from_secs(120),
            max_output_bytes: MAX_MANIFEST_BYTES as usize,
        })?,
        "PostgreSQL backup structural reinspection",
    )?;
    if observed_manifest.stdout != manifest_raw {
        return Err(OpsError::message(
            "PostgreSQL backup structural manifest differs on reinspection",
        ));
    }
    Ok(BackupClosure {
        dump,
        manifest,
        metadata: metadata_path,
        checksums,
        dump_sha256: metadata.dump_sha256,
        manifest_sha256: metadata.manifest_sha256,
        metadata_sha256: sha256_bytes(&metadata_raw),
        source_commit: metadata.source_commit,
        schema_identity: metadata.schema_identity,
        migration_identity: metadata.migration_identity,
    })
}

pub fn restore_into_fresh_database(
    config: &LkjmcConfig,
    backup_root: &Path,
    target_database: &str,
    expected_source_commit: &str,
) -> Result<()> {
    require_database_name(target_database)?;
    if target_database == config.database.database {
        return Err(OpsError::message(
            "restore target must differ from the configured live database",
        ));
    }
    let closure = verify_backup(config, backup_root, Some(expected_source_commit), 604_800)?;
    let metadata_raw = fs::read(&closure.metadata)
        .map_err(|error| OpsError::context("cannot reread backup metadata", error))?;
    let metadata: BackupMetadata = serde_json::from_slice(&metadata_raw)
        .map_err(|error| OpsError::context("invalid backup metadata", error))?;
    let mut target = connect(config, Some(target_database))?;
    let row = target
        .client
        .query_one(
            "select count(*)::bigint from pg_class c join pg_namespace n on n.oid=c.relnamespace where n.nspname not in ('pg_catalog','information_schema') and n.nspname not like 'pg_toast%' and c.relkind in ('r','p','v','m','S')",
            &[],
        )
        .map_err(|_| OpsError::message("cannot inspect restore target"))?;
    let count: i64 = row.get(0);
    if count != 0 {
        return Err(OpsError::message("restore target is not a fresh database"));
    }
    let output = run_bounded(&postgres_command(
        Path::new(PG_RESTORE),
        config,
        &target.password,
        vec![
            "--exit-on-error".to_string(),
            "--no-owner".to_string(),
            "--no-privileges".to_string(),
            "--dbname".to_string(),
            target_database.to_string(),
            path_text(&closure.dump)?,
        ],
        Duration::from_secs(900),
    ))?;
    require_success(output, "PostgreSQL restore")?;
    let restored_migrations = migration_marker(&mut target.client)?;
    if restored_migrations != metadata.migration_marker
        || migration_identity(&restored_migrations)? != metadata.migration_identity
        || schema_identity(&mut target.client)? != metadata.schema_identity
    {
        return Err(OpsError::message(
            "restored PostgreSQL schema or migration identity differs",
        ));
    }
    Ok(())
}

fn postgres_command(
    executable: &Path,
    config: &LkjmcConfig,
    password: &str,
    arguments: Vec<String>,
    timeout: Duration,
) -> CommandSpec {
    let mut complete = vec![
        "--host".to_string(),
        config.database.host.clone(),
        "--port".to_string(),
        config.database.port.to_string(),
        "--username".to_string(),
        config.database.user.clone(),
        "--no-password".to_string(),
    ];
    complete.extend(arguments);
    CommandSpec {
        executable: executable.to_path_buf(),
        arguments: complete,
        environment: BTreeMap::from([
            ("PGPASSWORD".to_string(), password.to_string()),
            ("PGCONNECT_TIMEOUT".to_string(), "10".to_string()),
        ]),
        stdin: Vec::new(),
        timeout,
        max_output_bytes: MAX_MANIFEST_BYTES as usize,
    }
}

fn schema_identity(client: &mut Client) -> Result<String> {
    let query = r#"
select jsonb_build_object(
  'columns', coalesce((select jsonb_agg(jsonb_build_object(
    'schema', table_schema, 'table', table_name, 'column', column_name,
    'position', ordinal_position, 'type', data_type, 'nullable', is_nullable,
    'default', column_default) order by table_schema, table_name, ordinal_position)
    from information_schema.columns
    where table_schema not in ('pg_catalog', 'information_schema')), '[]'::jsonb),
  'constraints', coalesce((select jsonb_agg(jsonb_build_object(
    'schema', n.nspname, 'table', c.relname, 'name', x.conname,
    'type', x.contype, 'definition', pg_get_constraintdef(x.oid, true))
    order by n.nspname, c.relname, x.conname)
    from pg_constraint x join pg_class c on c.oid=x.conrelid
    join pg_namespace n on n.oid=c.relnamespace
    where n.nspname not in ('pg_catalog', 'information_schema')), '[]'::jsonb),
  'indexes', coalesce((select jsonb_agg(jsonb_build_object(
    'schema', schemaname, 'table', tablename, 'name', indexname,
    'definition', indexdef) order by schemaname, tablename, indexname)
    from pg_indexes where schemaname not in ('pg_catalog', 'information_schema')), '[]'::jsonb)
)::text
"#;
    let value: String = client
        .query_one(query, &[])
        .map_err(|_| OpsError::message("cannot derive PostgreSQL schema identity"))?
        .get(0);
    Ok(sha256_bytes(value.as_bytes()))
}

fn migration_identity(marker: &[MigrationIdentity]) -> Result<String> {
    Ok(sha256_bytes(&serde_json::to_vec(marker)?))
}

fn validate_metadata(
    metadata: &BackupMetadata,
    expected_source_commit: Option<&str>,
    max_age_seconds: u64,
) -> Result<()> {
    if metadata.schema_version != 1 {
        return Err(OpsError::message("unsupported backup metadata schema"));
    }
    require_hex(&metadata.source_commit, 40, "backup source commit")?;
    if expected_source_commit.is_some_and(|value| value != metadata.source_commit) {
        return Err(OpsError::message("backup source commit differs"));
    }
    for (value, label) in [
        (&metadata.migration_identity, "backup migration identity"),
        (&metadata.schema_identity, "backup schema identity"),
        (&metadata.dump_sha256, "backup dump SHA-256"),
        (&metadata.manifest_sha256, "backup manifest SHA-256"),
    ] {
        require_hex(value, 64, label)?;
    }
    if metadata.server_version.is_empty() || metadata.server_version.len() > 128 {
        return Err(OpsError::message("backup server version is invalid"));
    }
    for migration in &metadata.migration_marker {
        validate_migration(migration)?;
    }
    if migration_identity(&metadata.migration_marker)? != metadata.migration_identity {
        return Err(OpsError::message("backup migration identity differs"));
    }
    let now = unix_seconds()?;
    if metadata.created_at_unix_seconds > now
        || now - metadata.created_at_unix_seconds > max_age_seconds
    {
        return Err(OpsError::message(
            "backup is outside the accepted age bound",
        ));
    }
    Ok(())
}

fn validate_migration(value: &MigrationIdentity) -> Result<()> {
    if value.version == 0 || value.name.is_empty() || value.name.len() > 256 {
        return Err(OpsError::message("invalid migration identity"));
    }
    require_hex(&value.checksum, 64, "migration checksum")
}

fn parse_secret(bytes: &[u8]) -> Result<String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| OpsError::message("database secret is not UTF-8"))?;
    if text.contains('\0') || text.lines().count() != 1 || text.trim().is_empty() {
        return Err(OpsError::message(
            "database secret must contain one nonempty line",
        ));
    }
    Ok(text.trim_end_matches(['\r', '\n']).to_string())
}

fn require_database_name(value: &str) -> Result<()> {
    if !value.is_empty()
        && value.len() <= 63
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && value.as_bytes()[0].is_ascii_lowercase()
    {
        Ok(())
    } else {
        Err(OpsError::message("PostgreSQL database name is unsafe"))
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

fn path_text(path: &Path) -> Result<String> {
    path.to_str()
        .map(ToString::to_string)
        .ok_or_else(|| OpsError::message("operational path is not UTF-8"))
}

fn sibling(path: &Path, prefix: &str) -> Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| OpsError::message("operation path has no parent"))?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| OpsError::message("operation path name is not UTF-8"))?;
    Ok(parent.join(format!("{prefix}-{name}-{}", Uuid::new_v4())))
}

fn remove_owned_stage(stage: &Path, destination: &Path) -> Result<()> {
    if stage.parent() != destination.parent()
        || !stage
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.starts_with(".lkjmc-backup-"))
    {
        return Err(OpsError::message("refusing to remove unowned backup state"));
    }
    let metadata = fs::symlink_metadata(stage)
        .map_err(|error| OpsError::context("cannot inspect partial backup", error))?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(OpsError::message("partial backup identity is ambiguous"));
    }
    fs::remove_dir_all(stage)
        .map_err(|error| OpsError::context("cannot remove partial backup", error))
}

fn unix_seconds() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|error| OpsError::context("system clock is before Unix epoch", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lkjmc_core::instance::{DesiredState, InstanceKind, ObservedState};

    #[test]
    fn migration_054_aligns_every_rust_kind_and_desired_state() {
        let migration =
            include_str!("../../../migrations/054-align-instance-kind-and-desired-state.sql");
        for (_, value) in InstanceKind::ALL {
            assert!(
                migration.contains(&format!("'{value}'")),
                "missing kind {value}"
            );
        }
        for (_, value) in DesiredState::ALL {
            assert!(
                migration.contains(&format!("'{value}'")),
                "missing desired state {value}"
            );
        }
    }

    #[test]
    fn observed_state_migration_aligns_every_rust_value() -> Result<()> {
        let base = include_str!("../../../migrations/002-instances.sql");
        let extension = include_str!("../../../migrations/036-runtime-observed-states.sql");
        let combined = format!("{base}\n{extension}");
        let values = [
            (ObservedState::ProcessAbsent, "process-absent"),
            (ObservedState::ProcessStarting, "process-starting"),
            (ObservedState::ProcessHealthy, "process-healthy"),
            (ObservedState::ProcessUnhealthy, "process-unhealthy"),
            (ObservedState::ProcessExited, "process-exited"),
            (ObservedState::ProcessUnknown, "process-unknown"),
            (ObservedState::KubernetesAbsent, "kubernetes-absent"),
            (ObservedState::KubernetesStarting, "kubernetes-starting"),
            (ObservedState::KubernetesReady, "kubernetes-ready"),
            (ObservedState::KubernetesUnhealthy, "kubernetes-unhealthy"),
            (ObservedState::KubernetesExited, "kubernetes-exited"),
            (ObservedState::KubernetesUnknown, "kubernetes-unknown"),
            (ObservedState::RuntimeAbsent, "runtime-absent"),
            (ObservedState::RuntimeStarting, "runtime-starting"),
            (ObservedState::RuntimeReady, "runtime-ready"),
            (ObservedState::RuntimeUnhealthy, "runtime-unhealthy"),
            (ObservedState::RuntimeExited, "runtime-exited"),
            (ObservedState::RuntimeUnknown, "runtime-unknown"),
        ];
        let mut seen = BTreeSet::new();
        for (state, value) in values {
            assert!(seen.insert(value));
            let _ = state;
            if !combined.contains(&format!("'{value}'")) {
                return Err(OpsError::message(format!(
                    "observed state migration is missing {value}"
                )));
            }
        }
        Ok(())
    }

    #[test]
    fn database_names_and_secrets_are_strict() {
        assert!(require_database_name("lkjmc_restore_1").is_ok());
        assert!(require_database_name("LKJMC").is_err());
        assert!(parse_secret(b"opaque-password\n").is_ok());
        assert!(parse_secret(b"first\nsecond\n").is_err());
    }
}
