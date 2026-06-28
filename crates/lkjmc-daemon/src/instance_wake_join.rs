use lkjmc_core::command::{ActorKind, CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::audit_helpers::audit;
use crate::instance_helpers::{body_string, runtime_running, start_runtime, store, with_client};

pub fn request(state: &AppState, envelope: CommandEnvelope) -> CommandResponse {
    with_client(state, envelope, |state, envelope, client| {
        let player_uuid = parse_uuid(&envelope, "playerUuid")?;
        let player_name = body_string(&envelope.body, "playerName")?;
        let target = body_string(&envelope.body, "targetInstanceId")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &player_name,
        ))?;
        let queue_id = Uuid::new_v4();
        store(lkjmc_store::wake_join::create(
            client,
            lkjmc_store::wake_join::NewWakeJoin {
                id: queue_id,
                player_uuid,
                player_name: &player_name,
                target_instance_id: &target,
                requested_by_kind: actor_kind(envelope.actor.kind),
                requested_by_name: &envelope.actor.name,
                expires_in_seconds: 60,
                metadata: json!({}),
            },
        ))?;
        store(lkjmc_store::wake_join::mark_waking(client, queue_id))?;
        match wake_target(state, client, &target) {
            Ok(()) => succeed(client, envelope, queue_id, &target),
            Err(error) => fail(client, envelope, queue_id, &target, error),
        }
    })
}

fn wake_target(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
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
    if observation.healthy {
        return Ok(());
    }
    Err(observation
        .message
        .unwrap_or_else(|| "wake start failed".to_string()))
}

fn succeed(
    client: &mut postgres::Client,
    envelope: CommandEnvelope,
    queue_id: Uuid,
    target: &str,
) -> Result<CommandResponse, String> {
    store(lkjmc_store::wake_join::mark_ready(client, queue_id, target))?;
    audit(
        client,
        &envelope,
        "instance.wake.request",
        "instance",
        target,
        "succeeded",
    )?;
    Ok(api::ok(
        envelope,
        json!({
            "queueId": queue_id.to_string(),
            "targetServer": target,
            "state": "ready"
        }),
    ))
}

fn fail(
    client: &mut postgres::Client,
    envelope: CommandEnvelope,
    queue_id: Uuid,
    target: &str,
    error: String,
) -> Result<CommandResponse, String> {
    store(lkjmc_store::wake_join::mark_failed(
        client, queue_id, &error,
    ))?;
    audit(
        client,
        &envelope,
        "instance.wake.request",
        "instance",
        target,
        "failed",
    )?;
    Err(error)
}

fn parse_uuid(envelope: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&envelope.body, field)?).map_err(|error| error.to_string())
}

fn actor_kind(kind: ActorKind) -> &'static str {
    match kind {
        ActorKind::Cli => "cli",
        ActorKind::VelocityPlugin => "velocity-plugin",
        ActorKind::PaperPlugin => "paper-plugin",
        ActorKind::Daemon => "daemon",
        ActorKind::Installer => "installer",
    }
}
