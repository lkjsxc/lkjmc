use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use lkjmc_core::config::LkjmcConfig;
use lkjmc_ops::{database, OpsError, Result};
use postgres::config::Host;
use postgres::{Config, NoTls};
use uuid::Uuid;

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Result<Self> {
        let path = std::env::temp_dir().join(format!("lkjmc-ops-postgres-{}", Uuid::new_v4()));
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

struct TestDatabases {
    base: Config,
    names: Vec<String>,
}

impl TestDatabases {
    fn create(base: Config, names: Vec<String>) -> Result<Self> {
        for name in &names {
            require_generated_database_name(name)?;
        }
        let mut control = base
            .connect(NoTls)
            .map_err(|_| OpsError::message("cannot connect to PostgreSQL test control database"))?;
        let mut created = Vec::new();
        for name in &names {
            if control
                .batch_execute(&format!("create database {name}"))
                .is_err()
            {
                for prior in created.iter().rev() {
                    let _ = control
                        .batch_execute(&format!("drop database if exists {prior} with (force)"));
                }
                return Err(OpsError::message(
                    "cannot create isolated PostgreSQL test database",
                ));
            }
            created.push(name.clone());
        }
        Ok(Self { base, names })
    }
}

impl Drop for TestDatabases {
    fn drop(&mut self) {
        let Ok(mut control) = self.base.connect(NoTls) else {
            return;
        };
        for name in &self.names {
            let _ = control.batch_execute(&format!("drop database if exists {name} with (force)"));
        }
    }
}

#[test]
fn backup_and_restore_verify_a_fresh_isolated_database() -> Result<()> {
    let Ok(base_url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        eprintln!("SKIP ops backup/restore: LKJMC_STORE_TEST_DATABASE_URL is unset");
        return Ok(());
    };
    let base: Config = base_url
        .parse()
        .map_err(|_| OpsError::message("invalid PostgreSQL test connection configuration"))?;
    let source_database = format!("lkjmc_ops_source_{}", Uuid::new_v4().simple());
    let target_database = format!("lkjmc_ops_restore_{}", Uuid::new_v4().simple());
    let databases = TestDatabases::create(
        base.clone(),
        vec![source_database.clone(), target_database.clone()],
    )?;
    let root = TestRoot::new()?;
    let secret = root.0.join("database.secret");
    write_database_secret(&base, &secret)?;

    let mut config: LkjmcConfig =
        serde_json::from_str(include_str!("../../../config/defaults/daemon.json.example"))?;
    config.database.host = tcp_host(&base)?;
    config.database.port = base.get_ports().first().copied().unwrap_or(5432);
    config.database.user = base
        .get_user()
        .ok_or_else(|| OpsError::message("PostgreSQL test configuration has no user"))?
        .to_string();
    config.database.database.clone_from(&source_database);
    config.database.secret_file = path_text(&secret)?;

    database::apply_migrations(&config)?;
    let mut source = database::connect(&config, None)?;
    source
        .client
        .execute(
            "insert into instances (id, kind, desired_state, config) values ($1, $2, $3, $4)",
            &[&"ember-realm", &"paper", &"running", &serde_json::json!({})],
        )
        .map_err(|_| OpsError::message("cannot seed PostgreSQL backup fixture"))?;
    drop(source);

    let backup = root.0.join("accepted-backup");
    let source_commit = "a".repeat(40);
    let closure = database::create_backup(&config, &backup, &source_commit)?;
    assert_eq!(closure.source_commit, source_commit);
    database::restore_into_fresh_database(&config, &backup, &target_database, &source_commit)?;
    let mut restored = database::connect(&config, Some(&target_database))?;
    let restored_id: String = restored
        .client
        .query_one("select id from instances", &[])
        .map_err(|_| OpsError::message("cannot inspect restored PostgreSQL data"))?
        .get(0);
    assert_eq!(restored_id, "ember-realm");
    drop(restored);

    fs::write(&closure.manifest, b"changed\n")?;
    assert!(database::verify_backup(&config, &backup, Some(&source_commit), 3600).is_err());
    drop(databases);
    Ok(())
}

fn write_database_secret(base: &Config, path: &Path) -> Result<()> {
    let password = base
        .get_password()
        .ok_or_else(|| OpsError::message("PostgreSQL test configuration has no password"))?;
    let mut bytes = password.to_vec();
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

fn tcp_host(base: &Config) -> Result<String> {
    match base.get_hosts().first() {
        Some(Host::Tcp(host)) => Ok(host.clone()),
        _ => Err(OpsError::message(
            "PostgreSQL operations test requires one TCP host",
        )),
    }
}

fn require_generated_database_name(value: &str) -> Result<()> {
    if value.len() <= 63
        && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        Ok(())
    } else {
        Err(OpsError::message(
            "generated PostgreSQL test database name is unsafe",
        ))
    }
}

fn path_text(path: &Path) -> Result<String> {
    path.to_str()
        .map(ToString::to_string)
        .ok_or_else(|| OpsError::message("PostgreSQL test path is not UTF-8"))
}
