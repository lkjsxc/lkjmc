mod fixture;
mod recovery_matrix;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use lkjmc_core::network_intent::{ChangeAction, InspectionOutcome};
use serde_json::json;

use fixture::Fixture;

#[test]
fn network_apply_real_local_boundary_and_reapply() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    let decoy = fixture.seed_newer_folia_decoy()?;
    let first = super::apply(&fixture.state, request("network-probe-apply")?);
    if !first.ok {
        let log = std::fs::read_to_string(fixture.root.join("logs/hub/current.log"))
            .unwrap_or_else(|error| format!("log unavailable: {error}"));
        return Err(format!(
            "first apply failed: {:?}; hub log: {log}",
            first.error
        ));
    }
    assert_eq!(
        super::super::network_state::inspect(&fixture.state)?.outcome,
        InspectionOutcome::NoOp
    );
    for id in ["hub", "proxy", "survival"] {
        assert!(crate::runtime::process::group_exists(fixture.pid(id)?));
    }
    for id in ["hub", "survival"] {
        let properties =
            std::fs::read_to_string(fixture.root.join("data").join(id).join("server.properties"))
                .map_err(|error| error.to_string())?;
        assert!(properties.lines().any(|line| line == "server-ip=127.0.0.1"));
    }
    let configured_folia = fixture
        .config
        .network
        .assets
        .iter()
        .find(|asset| asset.id == "folia-server")
        .ok_or("configured Folia asset missing")?
        .path
        .clone();
    assert_eq!(fixture.selected_jar_path("hub")?, configured_folia);
    assert_eq!(fixture.selected_jar_path("survival")?, configured_folia);
    let old_hub_pid = fixture.pid("hub")?;
    fixture.bind_instance_to_jar("hub", decoy)?;
    let drift = super::super::network_state::inspect(&fixture.state)?;
    assert_eq!(drift.outcome, InspectionOutcome::Changes);
    let stop_order = drift
        .changes
        .iter()
        .position(|change| {
            change.instance_id.as_deref() == Some("hub") && change.action == ChangeAction::Stop
        })
        .ok_or("asset-drift stop missing")?;
    let render_order = drift
        .changes
        .iter()
        .position(|change| {
            change.instance_id.as_deref() == Some("hub") && change.action == ChangeAction::Render
        })
        .ok_or("asset-drift render missing")?;
    assert!(stop_order < render_order);
    let interrupted = fixture.seed_attempt("network-asset-drift-interrupted", "runtime")?;
    crate::support::instance_helpers::stop_runtime(&fixture.state, "hub")?;
    assert!(!crate::runtime::process::group_exists(old_hub_pid));
    let repaired = super::apply(&fixture.state, request("network-asset-drift-repair")?);
    if !repaired.ok {
        return Err(format!("asset drift repair failed: {:?}", repaired.error));
    }
    assert_ne!(fixture.pid("hub")?, old_hub_pid);
    assert_eq!(fixture.selected_jar_path("hub")?, configured_folia);
    let interrupted = fixture.attempt(interrupted)?;
    assert_eq!(interrupted.outcome, "unknown");
    assert_eq!(interrupted.observation["recoveryComplete"], true);
    let velocity = std::fs::read_to_string(fixture.root.join("data/proxy/velocity.toml"))
        .map_err(|error| error.to_string())?;
    assert!(velocity.contains("hub = \"127.0.0.1:"));
    assert!(velocity.contains("survival = \"127.0.0.1:"));
    assert!(velocity.contains("try = [\"hub\", \"survival\"]"));
    let second = super::apply(&fixture.state, request("network-probe-reapply")?);
    assert!(second.ok);
    assert_eq!(
        second
            .body
            .as_ref()
            .and_then(|body| body["result"].as_str()),
        Some("no-op")
    );
    let secret = Path::new(&fixture.config.network.forwarding.secret_file);
    assert_eq!(
        std::fs::metadata(secret)
            .map_err(|error| error.to_string())?
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let lock_path = fixture.root.join("data/.network-apply.lock");
    assert_eq!(
        std::fs::metadata(lock_path)
            .map_err(|error| error.to_string())?
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    lock_and_deadline(&fixture.root.join("lock-test"))
}

#[test]
fn network_reapply_retires_legacy_instance_environment() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    let first = super::apply(&fixture.state, request("network-config-schema-first")?);
    if !first.ok {
        return Err(format!("first apply failed: {:?}", first.error));
    }
    let old_pid = fixture.pid("hub")?;
    fixture.make_instance_config_legacy("hub")?;

    let drift = super::super::network_state::inspect(&fixture.state)?;
    assert_eq!(drift.outcome, InspectionOutcome::Changes);
    let stop_order = drift
        .changes
        .iter()
        .position(|change| {
            change.instance_id.as_deref() == Some("hub") && change.action == ChangeAction::Stop
        })
        .ok_or("legacy-config stop missing")?;
    let render_order = drift
        .changes
        .iter()
        .position(|change| {
            change.instance_id.as_deref() == Some("hub") && change.action == ChangeAction::Render
        })
        .ok_or("legacy-config render missing")?;
    assert!(stop_order < render_order);

    let repaired = super::apply(&fixture.state, request("network-config-schema-repair")?);
    if !repaired.ok {
        return Err(format!("legacy config repair failed: {:?}", repaired.error));
    }
    assert_ne!(fixture.pid("hub")?, old_pid);
    let config = fixture.instance_config("hub")?;
    assert_eq!(config["configSchemaVersion"], json!(2));
    assert_eq!(
        config["env"]["LKJMC_HEARTBEAT_ENDPOINT"],
        json!("http://127.0.0.1:8765/plugin/v1/heartbeat")
    );
    assert_eq!(
        config["env"]["LKJMC_HEARTBEAT_CREDENTIAL_FILE"],
        json!("/var/lib/lkjmc/private/plugin-credentials/hub.secret")
    );
    assert!(config["env"].get("LKJMC_DAEMON_HTTP_URL").is_none());
    Ok(())
}

#[test]
fn network_reapply_repairs_killed_owned_proxy() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    let first = super::apply(&fixture.state, request("network-kill-first")?);
    if !first.ok {
        return Err(format!("first apply failed: {:?}", first.error));
    }
    let old_pid = fixture.pid("proxy")?;
    let history = fixture.runtime_history_count("proxy")?;
    assert!(crate::runtime::process::kill_group(old_pid));
    let repaired = super::apply(&fixture.state, request("network-kill-repair")?);
    if !repaired.ok {
        return Err(format!("killed proxy repair failed: {:?}", repaired.error));
    }
    let new_pid = fixture.pid("proxy")?;
    assert_ne!(new_pid, old_pid);
    assert!(crate::runtime::process::group_exists(new_pid));
    assert!(fixture.runtime_history_count("proxy")? > history);
    let attempts = fixture.attempts_for("network-kill-repair")?;
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].outcome, "observed");
    Ok(())
}

#[test]
fn network_apply_denies_unowned_listener() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    let port = fixture
        .config
        .network
        .listeners
        .iter()
        .find(|listener| listener.id == "proxy-java")
        .ok_or("proxy listener missing")?
        .port;
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", port)).map_err(|error| error.to_string())?;
    let denied = super::apply(&fixture.state, request("network-unowned-listener")?);
    assert!(!denied.ok);
    assert!(listener.local_addr().is_ok());
    assert!(fixture.tracked_pids()?.is_empty());
    Ok(())
}

#[test]
fn network_apply_recovers_after_partial_process_failure() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("lkjmc-command-does-not-exist")? else {
        return Ok(());
    };
    let failed = super::apply(&fixture.state, request("network-probe-failure")?);
    assert!(!failed.ok);
    fixture.repair_proxy()?;
    let repaired = super::apply(&fixture.state, request("network-probe-repair")?);
    if !repaired.ok {
        return Err(format!("repair failed: {:?}", repaired.error));
    }
    let failed_history = fixture.attempts_for("network-probe-failure")?;
    let uncertain = failed_history.first().ok_or("failed attempt missing")?;
    assert_eq!(uncertain.outcome, "unknown");
    assert_eq!(uncertain.observation["recoveryComplete"], true);
    assert!(fixture
        .attempts_for("network-probe-repair")?
        .iter()
        .any(|attempt| matches!(attempt.outcome.as_str(), "observed" | "no-op")));
    Ok(())
}

pub(super) fn request(id: &str) -> Result<CommandEnvelope, String> {
    Ok(CommandEnvelope {
        request_id: CommandId::parse("request id", id).map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "network-probe".to_string(),
        },
        command: "bootstrap.apply".to_string(),
        body: json!({
            "profile":"playable",
            "acceptMinecraftEula":true,
            "bedrock":"disabled"
        }),
    })
}

fn lock_and_deadline(root: &Path) -> Result<(), String> {
    let first = super::lock::acquire_with_deadline(
        root.to_string_lossy().as_ref(),
        Duration::from_millis(5),
        Duration::from_secs(1),
    )?;
    assert!(super::lock::acquire_with_deadline(
        root.to_string_lossy().as_ref(),
        Duration::from_millis(5),
        Duration::from_secs(1)
    )
    .is_err());
    drop(first);
    let expired = super::lock::acquire_with_deadline(
        root.to_string_lossy().as_ref(),
        Duration::from_millis(5),
        Duration::ZERO,
    )?;
    assert!(expired.remaining().is_err());
    Ok(())
}
