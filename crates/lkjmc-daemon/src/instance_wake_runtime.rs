use lkjmc_core::command::{ActorKind, CommandEnvelope};

use crate::app::AppState;
use crate::instance_helpers::{runtime_running, start_runtime, store};

pub fn wake_target(
    state: &AppState,
    client: &mut postgres::Client,
    id: &str,
) -> Result<(), String> {
    let instance = store(lkjmc_store::instance::get(client, id))?
        .ok_or_else(|| format!("instance not found: {id}"))?;
    if !matches!(
        instance.desired_state.as_str(),
        "suspended" | "running" | "starting"
    ) {
        return Err(format!(
            "instance is not suspended or running: {}",
            instance.desired_state
        ));
    }
    store(lkjmc_store::instance_presence::clear_autosuspended(
        client, id,
    ))?;
    store(lkjmc_store::instance::update_desired_state(
        client, id, "running",
    ))?;
    if runtime_running(state, id)? {
        return Ok(());
    }
    let observation = start_runtime(state, client, id)?;
    observation.healthy.then_some(()).ok_or_else(|| {
        observation
            .message
            .unwrap_or_else(|| "wake start failed".to_string())
    })
}

pub fn actor_kind(kind: ActorKind) -> &'static str {
    match kind {
        ActorKind::Cli => "cli",
        ActorKind::VelocityPlugin => "velocity-plugin",
        ActorKind::PaperPlugin => "paper-plugin",
        ActorKind::Daemon => "daemon",
        ActorKind::Installer => "installer",
        ActorKind::WebOperator => "web-operator",
    }
}

pub fn ttl(envelope: &CommandEnvelope) -> i32 {
    envelope
        .body
        .get("expiresInSeconds")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(60)
        .clamp(5, 600) as i32
}
