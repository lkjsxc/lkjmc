mod fixture;
mod recovery_matrix;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use lkjmc_core::network_intent::InspectionOutcome;
use serde_json::json;

use fixture::Fixture;

#[test]
fn network_apply_real_local_boundary_and_reapply() -> Result<(), String> {
    let Some(fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    let first = super::apply(&fixture.state, request("network-probe-apply")?);
    if !first.ok {
        return Err(format!("first apply failed: {:?}", first.error));
    }
    assert_eq!(
        super::super::network_state::inspect(&fixture.state)?.outcome,
        InspectionOutcome::NoOp
    );
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
