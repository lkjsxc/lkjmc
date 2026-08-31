use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use lkjmc_ops::error::{OpsError, Result};
use serde::Serialize;
use serde_json::json;

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("lkjmc operations failed: {error}");
        std::process::exit(1);
    }
}

fn run(arguments: Vec<String>) -> Result<()> {
    match arguments.as_slice() {
        [command] if command == "version" => version(),
        [group, command, rest @ ..] if group == "release" && command == "verify" => {
            release_verify(rest)
        }
        [group, command, rest @ ..] if group == "artifacts" && command == "install" => {
            artifacts_install(rest)
        }
        [group, area, command, rest @ ..]
            if group == "eula" && area == "policy" && command == "create" =>
        {
            eula_policy_create(rest)
        }
        [group, command, rest @ ..] if group == "eula" && command == "materialize" => {
            eula_materialize(rest)
        }
        [group, command, rest @ ..] if group == "fence" && command == "check" => fence_check(rest),
        [group, command, rest @ ..] if group == "database" && command == "backup" => {
            database_backup(rest)
        }
        [group, command, rest @ ..] if group == "database" && command == "backup-verify" => {
            database_backup_verify(rest)
        }
        [group, command, rest @ ..] if group == "database" && command == "restore-verify" => {
            database_restore_verify(rest)
        }
        [group, command, rest @ ..] if group == "bootstrap" && command == "after-start" => {
            bootstrap_after_start(rest)
        }
        [group, command, rest @ ..] if group == "diagnose" && command == "inventory" => {
            diagnose_inventory(rest)
        }
        _ => Err(usage()),
    }
}

fn version() -> Result<()> {
    println!(
        "lkjmc-ops {} commit={} dirty={}",
        env!("CARGO_PKG_VERSION"),
        lkjmc_core::build_info::COMMIT,
        lkjmc_core::build_info::dirty_label()
    );
    Ok(())
}

fn release_verify(arguments: &[String]) -> Result<()> {
    let flags = Flags::parse(arguments, &["release-root", "manifest-sha256"])?;
    let release = lkjmc_ops::manifest::VerifiedRelease::load_anchored(
        &flags.path("release-root")?,
        flags.required("manifest-sha256")?,
    )?;
    output(&json!({
        "schemaVersion":1,"result":"release-verified","commit":release.manifest.commit,
        "manifestSha256":release.manifest_sha256,
        "artifacts":release.artifacts().map(|item| item.path.as_str()).collect::<Vec<_>>()
    }))
}

fn artifacts_install(arguments: &[String]) -> Result<()> {
    let flags = Flags::parse(
        arguments,
        &[
            "release-root",
            "manifest-sha256",
            "root",
            "scope",
            "service-uid",
            "service-gid",
        ],
    )?;
    let release = lkjmc_ops::manifest::VerifiedRelease::load_anchored(
        &flags.path("release-root")?,
        flags.required("manifest-sha256")?,
    )?;
    let scope = match flags.required("scope")? {
        "user" => lkjmc_ops::install::InstallScope::User,
        "system" => lkjmc_ops::install::InstallScope::System {
            service_uid: flags.u32("service-uid")?,
            service_gid: flags.u32("service-gid")?,
        },
        _ => return Err(OpsError::message("install scope must be user or system")),
    };
    let result = lkjmc_ops::install::install(
        &release,
        &flags.path("root")?,
        scope,
        lkjmc_ops::install::InstallFault::None,
    )?;
    output(
        &json!({"schemaVersion":1,"result":result,"commit":release.manifest.commit,"manifestSha256":release.manifest_sha256}),
    )
}

fn eula_policy_create(arguments: &[String]) -> Result<()> {
    lkjmc_ops::require_root()?;
    let flags = Flags::parse(arguments, &["policy", "service-gid"])?;
    let policy = flags.path("policy")?;
    let gid = flags.u32("service-gid")?;
    let changed = lkjmc_ops::eula::create_policy(&policy, 0, gid)?;
    output(&json!({
        "schemaVersion":1,"result":if changed {"created"} else {"no-op"},
        "policySha256":lkjmc_ops::eula::verify_policy(&policy,0,gid)?
    }))
}

fn eula_materialize(arguments: &[String]) -> Result<()> {
    lkjmc_ops::require_root()?;
    let flags = Flags::parse(
        arguments,
        &["config", "policy", "service-uid", "service-gid"],
    )?;
    let config = lkjmc_ops::fleet::read_config(&flags.path("config")?)?;
    let fleet = lkjmc_ops::fleet::FleetSnapshot::from_config(&config)?;
    output(&lkjmc_ops::eula::materialize(
        &fleet,
        &flags.path("policy")?,
        0,
        flags.u32("service-uid")?,
        flags.u32("service-gid")?,
    )?)
}

fn fence_check(arguments: &[String]) -> Result<()> {
    lkjmc_ops::require_root()?;
    let flags = Flags::parse(arguments, &["fence", "permit", "trusted-root"])?;
    let fence = flags
        .optional_path("fence")
        .unwrap_or_else(|| PathBuf::from("/etc/lkjmc/deployment-fence.json"));
    let permit = flags
        .optional_path("permit")
        .unwrap_or_else(|| PathBuf::from("/run/lkjmc-deploy-start-permit.json"));
    let trusted = flags
        .optional_path("trusted-root")
        .unwrap_or_else(|| PathBuf::from("/"));
    output(
        &json!({"schemaVersion":1,"result":lkjmc_ops::fence::check(&fence,&permit,&trusted,0,0)?}),
    )
}

fn database_backup(arguments: &[String]) -> Result<()> {
    lkjmc_ops::require_root()?;
    let flags = Flags::parse(arguments, &["config", "output", "source-commit"])?;
    let config = lkjmc_ops::fleet::read_config(&flags.path("config")?)?;
    let closure = lkjmc_ops::database::create_backup(
        &config,
        &flags.path("output")?,
        flags.required("source-commit")?,
    )?;
    output(&json!({
        "schemaVersion":1,"result":"backup-verified","sourceCommit":closure.source_commit,
        "dumpSha256":closure.dump_sha256,"manifestSha256":closure.manifest_sha256,
        "schemaIdentity":closure.schema_identity,"migrationIdentity":closure.migration_identity
    }))
}

fn database_backup_verify(arguments: &[String]) -> Result<()> {
    lkjmc_ops::require_root()?;
    let flags = Flags::parse(
        arguments,
        &["config", "backup", "source-commit", "max-age-seconds"],
    )?;
    let config = lkjmc_ops::fleet::read_config(&flags.path("config")?)?;
    let closure = lkjmc_ops::database::verify_backup(
        &config,
        &flags.path("backup")?,
        Some(flags.required("source-commit")?),
        flags.u64_or("max-age-seconds", 3600)?,
    )?;
    output(
        &json!({"schemaVersion":1,"result":"backup-verified","sourceCommit":closure.source_commit,"dumpSha256":closure.dump_sha256,"manifestSha256":closure.manifest_sha256}),
    )
}

fn database_restore_verify(arguments: &[String]) -> Result<()> {
    lkjmc_ops::require_root()?;
    let flags = Flags::parse(
        arguments,
        &["config", "backup", "source-commit", "target-database"],
    )?;
    let config = lkjmc_ops::fleet::read_config(&flags.path("config")?)?;
    lkjmc_ops::database::restore_into_fresh_database(
        &config,
        &flags.path("backup")?,
        flags.required("target-database")?,
        flags.required("source-commit")?,
    )?;
    output(
        &json!({"schemaVersion":1,"result":"restore-verified","sourceCommit":flags.required("source-commit")?,"targetDatabase":flags.required("target-database")?}),
    )
}

fn bootstrap_after_start(arguments: &[String]) -> Result<()> {
    let flags = Flags::parse(
        arguments,
        &["config", "cli", "expected-commit", "socket-timeout-seconds"],
    )?;
    let expected = flags
        .optional("expected-commit")
        .unwrap_or(lkjmc_core::build_info::COMMIT);
    output(&lkjmc_ops::bootstrap::after_start(
        &flags
            .optional_path("config")
            .unwrap_or_else(|| PathBuf::from("/etc/lkjmc/lkjmc.json")),
        &flags
            .optional_path("cli")
            .unwrap_or_else(|| PathBuf::from("/opt/lkjmc/releases/current/bin/lkjmc")),
        expected,
        Duration::from_secs(flags.u64_or("socket-timeout-seconds", 120)?),
    )?)
}

fn diagnose_inventory(arguments: &[String]) -> Result<()> {
    let flags = Flags::parse(
        arguments,
        &["config", "persisted", "status", "expected-commit"],
    )?;
    let config = lkjmc_ops::fleet::read_config(&flags.path("config")?)?;
    let fleet = lkjmc_ops::fleet::FleetSnapshot::from_config(&config)?;
    let persisted: Vec<lkjmc_ops::fleet::PersistedInstance> =
        read_json(&flags.path("persisted")?, "persisted inventory")?;
    fleet.compare_persisted(&persisted)?;
    let status: serde_json::Value = read_json(&flags.path("status")?, "status inventory")?;
    fleet.validate_status(&status, flags.required("expected-commit")?)?;
    output(&json!({
        "schemaVersion":1,"result":"inventory-agrees","fleetRevision":fleet.revision,
        "instanceIds":fleet.instances().map(|item|item.id.as_str()).collect::<Vec<_>>(),
        "velocityInstanceId":fleet.velocity_entry()?.id.as_str(),
        "pluginInstanceIds":fleet.plugin_targets().iter().map(|item|item.instance_id.as_str()).collect::<Vec<_>>(),
        "credentialInstanceIds":fleet.credential_targets().iter().map(|item|item.instance_id.as_str()).collect::<Vec<_>>(),
        "eulaInstanceIds":fleet.eula_targets().iter().map(|item|item.instance_id.as_str()).collect::<Vec<_>>()
    }))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T> {
    let bytes = fs::read(path)
        .map_err(|error| OpsError::context(&format!("cannot read {label}"), error))?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(OpsError::message(format!("{label} exceeds four MiB")));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| OpsError::context(&format!("invalid {label}"), error))
}

fn output(value: &impl Serialize) -> Result<()> {
    let text = serde_json::to_string(value)
        .map_err(|error| OpsError::context("cannot serialize operation receipt", error))?;
    println!("{text}");
    Ok(())
}

fn usage() -> OpsError {
    OpsError::message("usage: lkjmc-ops version|release verify|artifacts install|deploy update|deploy recover|database backup|database backup-verify|database restore-verify|eula policy create|eula materialize|fence check|bootstrap after-start|diagnose inventory")
}

struct Flags {
    values: BTreeMap<String, String>,
}

impl Flags {
    fn parse(arguments: &[String], allowed: &[&str]) -> Result<Self> {
        let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
        let mut values = BTreeMap::new();
        let mut index = 0;
        while index < arguments.len() {
            let flag = arguments[index].strip_prefix("--").ok_or_else(|| {
                OpsError::message(format!("expected flag, got {}", arguments[index]))
            })?;
            if !allowed.contains(flag) {
                return Err(OpsError::message(format!("unknown flag: --{flag}")));
            }
            let value = arguments
                .get(index + 1)
                .ok_or_else(|| OpsError::message(format!("missing value for --{flag}")))?;
            if value.starts_with("--") {
                return Err(OpsError::message(format!("missing value for --{flag}")));
            }
            if values.insert(flag.to_string(), value.clone()).is_some() {
                return Err(OpsError::message(format!("duplicate flag: --{flag}")));
            }
            index += 2;
        }
        Ok(Self { values })
    }
    fn required(&self, name: &str) -> Result<&str> {
        self.values
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| OpsError::message(format!("missing required flag: --{name}")))
    }
    fn optional(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }
    fn path(&self, name: &str) -> Result<PathBuf> {
        Ok(PathBuf::from(self.required(name)?))
    }
    fn optional_path(&self, name: &str) -> Option<PathBuf> {
        self.optional(name).map(PathBuf::from)
    }
    fn u32(&self, name: &str) -> Result<u32> {
        self.required(name)?
            .parse()
            .map_err(|_| OpsError::message(format!("--{name} must be an unsigned integer")))
    }
    fn u64_or(&self, name: &str, default: u64) -> Result<u64> {
        match self.optional(name) {
            Some(value) => value
                .parse()
                .map_err(|_| OpsError::message(format!("--{name} must be an unsigned integer"))),
            None => Ok(default),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags_reject_unknown_duplicate_and_missing_values() {
        assert!(Flags::parse(&["--unknown".into(), "x".into()], &["known"]).is_err());
        assert!(Flags::parse(
            &["--known".into(), "x".into(), "--known".into(), "y".into()],
            &["known"]
        )
        .is_err());
        assert!(Flags::parse(&["--known".into()], &["known"]).is_err());
    }
}
