use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::{body_string, store, with_connection};

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "instance.wake.request" => request_wake(state, request),
        "instance.wake.status" => status(state, request),
        "instance.wake.cancel" => cancel(state, request),
        "instance.wake.consume" => consume(state, request),
        "instance.wake.cleanup" => cleanup(state, request),
        _ => api::error(request, "command.unknown", "unknown wake command", false),
    }
}

fn request_wake(state: &AppState, envelope: CommandEnvelope) -> CommandResponse {
    let result = (|| {
        let player_uuid = parse_uuid(&envelope, "playerUuid")?;
        let player_name = body_string(&envelope.body, "playerName")?;
        let target = body_string(&envelope.body, "targetInstanceId")?;
        let queue_id = Uuid::new_v4();
        {
            let mut client = state.database_connection()?;
            store(lkjmc_store::player::insert_identity(
                &mut *client,
                player_uuid,
                &player_name,
            ))?;
            let row = store(lkjmc_store::wake_join::create_or_live(
                &mut client,
                lkjmc_store::wake_join::NewWakeJoin {
                    id: queue_id,
                    player_uuid,
                    player_name: &player_name,
                    target_instance_id: &target,
                    requested_by_kind: crate::commands::instance_wake_runtime::actor_kind(
                        envelope.actor.kind,
                    ),
                    requested_by_name: &envelope.actor.name,
                    expires_in_seconds: crate::commands::instance_wake_runtime::ttl(&envelope),
                    correlation_id: envelope.request_id.as_str(),
                    metadata: json!({}),
                },
            ))?;
            if row.id != queue_id || row.state == "ready" {
                return Ok(response(envelope.clone(), &row));
            }
            store(lkjmc_store::wake_join::mark_starting(&mut client, queue_id))?;
        }
        let effect = crate::commands::instance_wake_runtime::wake_target(state, &target);
        let mut client = state.database_connection()?;
        match effect {
            Ok(()) => succeed(&mut client, envelope.clone(), queue_id, &target),
            Err(error) => fail(&mut client, envelope.clone(), queue_id, &target, error),
        }
    })();
    result.unwrap_or_else(|error| api::error(envelope, "instance.wake.error", error, false))
}

fn status(state: &AppState, envelope: CommandEnvelope) -> CommandResponse {
    with_connection(state, envelope, |_state, envelope, client| {
        let id = queue_id(&envelope)?;
        let row = store(lkjmc_store::wake_join::get(client, id))?
            .ok_or_else(|| "wake request not found".to_string())?;
        Ok(response(envelope, &row))
    })
}

fn cancel(state: &AppState, envelope: CommandEnvelope) -> CommandResponse {
    with_connection(state, envelope, |_state, envelope, client| {
        let id = queue_id(&envelope)?;
        let player_uuid = parse_uuid(&envelope, "playerUuid")?;
        let row = store(lkjmc_store::wake_join::cancel(client, id, player_uuid))?
            .ok_or_else(|| "wake request not found".to_string())?;
        audit(
            client,
            &envelope,
            "instance.wake.cancel",
            "wake-request",
            &id.to_string(),
            "succeeded",
        )?;
        Ok(response(envelope, &row))
    })
}

fn consume(state: &AppState, envelope: CommandEnvelope) -> CommandResponse {
    with_connection(state, envelope, |_state, envelope, client| {
        let id = queue_id(&envelope)?;
        let target = body_string(&envelope.body, "targetServer")?;
        let row = store(lkjmc_store::wake_join::consume_ready(client, id, &target))?
            .ok_or_else(|| "wake request is not ready".to_string())?;
        audit(
            client,
            &envelope,
            "instance.wake.consume",
            "wake-request",
            &id.to_string(),
            "succeeded",
        )?;
        Ok(response(envelope, &row))
    })
}

fn cleanup(state: &AppState, envelope: CommandEnvelope) -> CommandResponse {
    with_connection(state, envelope, |_state, envelope, client| {
        let expired = store(lkjmc_store::wake_join::expire_due(client))?;
        Ok(api::ok(envelope, json!({"expired": expired})))
    })
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
    status_row(client, envelope, queue_id)
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

fn status_row(
    client: &mut postgres::Client,
    envelope: CommandEnvelope,
    id: Uuid,
) -> Result<CommandResponse, String> {
    let row = store(lkjmc_store::wake_join::get(client, id))?
        .ok_or_else(|| "wake request not found".to_string())?;
    Ok(response(envelope, &row))
}

fn response(
    envelope: CommandEnvelope,
    row: &lkjmc_store::wake_join::WakeJoinRecord,
) -> CommandResponse {
    api::ok(
        envelope,
        json!({"queueId": row.id.to_string(), "targetServer": row.target_server.clone().unwrap_or_else(|| row.target_instance_id.clone()), "state": row.state, "failureReason": row.failure_reason}),
    )
}

fn parse_uuid(envelope: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&envelope.body, field)?).map_err(|error| error.to_string())
}

fn queue_id(envelope: &CommandEnvelope) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&envelope.body, "queueId")?).map_err(|error| error.to_string())
}
