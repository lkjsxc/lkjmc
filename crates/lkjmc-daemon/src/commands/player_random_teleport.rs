use lkjmc_core::command::CommandEnvelope;
use lkjmc_core::random_teleport::{RandomTeleportDecision, RandomTeleportPolicy};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn quote(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let server_id = body_string(&request.body, "serverId")?;
        let policy = profile_policy(&request)?;
        let balance = store(lkjmc_store::points::balance(client, player_uuid))?;
        let remaining = store(lkjmc_store::random_teleport::cooldown_remaining(
            client,
            player_uuid,
            &server_id,
            &policy.profile_id,
            policy.cooldown_seconds,
        ))?;
        Ok(api::ok(request, quote_body(&policy, balance, remaining)))
    })
}

pub fn reserve(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let server_id = body_string(&request.body, "serverId")?;
        let world = body_string(&request.body, "world")?;
        let correlation_id = parse_uuid(&request, "correlationId")?;
        let policy = profile_policy(&request)?;
        let remaining = store(lkjmc_store::random_teleport::cooldown_remaining(
            client,
            player_uuid,
            &server_id,
            &policy.profile_id,
            policy.cooldown_seconds,
        ))?;
        if let Some(error) = decision_error(&request, policy.decide(remaining)) {
            return Ok(error);
        }
        if !policy.world_allowed(&world) {
            return Ok(api::error(
                request,
                "rtp.disabled",
                "world is not enabled for random teleport",
                false,
            ));
        }
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        let outcome = store(lkjmc_store::random_teleport::reserve(
            client,
            lkjmc_store::random_teleport::ReserveInput {
                id: Uuid::new_v4(),
                correlation_id,
                player_uuid,
                server_id: &server_id,
                profile_id: &policy.profile_id,
                world: &world,
                x: number(&request, "x")?,
                y: number(&request, "y")?,
                z: number(&request, "z")?,
                cost_points: policy.cost_points,
            },
        ))?;
        match outcome {
            lkjmc_store::random_teleport::ReserveOutcome::Reserved => Ok(api::ok(
                request,
                json!({"state":"reserved","correlationId":correlation_id.to_string(),"costPoints":policy.cost_points}),
            )),
            lkjmc_store::random_teleport::ReserveOutcome::Existing(state) => Ok(api::ok(
                request,
                json!({"state":state,"correlationId":correlation_id.to_string(),"costPoints":policy.cost_points}),
            )),
            lkjmc_store::random_teleport::ReserveOutcome::InsufficientPoints => Ok(api::error(
                request,
                "rtp.insufficient_points",
                "insufficient points for random teleport",
                false,
            )),
        }
    })
}

pub fn complete(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let correlation_id = parse_uuid(&request, "correlationId")?;
        let completed = store(lkjmc_store::random_teleport::complete(
            client,
            player_uuid,
            correlation_id,
        ))?;
        Ok(api::ok(request, json!({"completed": completed})))
    })
}

pub fn refund(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let correlation_id = parse_uuid(&request, "correlationId")?;
        let reason = body_string(&request.body, "reason")?;
        let refunded = store(lkjmc_store::random_teleport::refund(
            client,
            player_uuid,
            correlation_id,
            &reason,
        ))?;
        Ok(api::ok(request, json!({"refunded": refunded})))
    })
}

pub fn history(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let records = store(lkjmc_store::random_teleport::history(client, player_uuid))?;
        let history = records
            .into_iter()
            .map(|item| {
                json!({
                    "correlationId": item.correlation_id.to_string(), "serverId": item.server_id,
                    "profileId": item.profile_id, "world": item.world,
                    "x": item.x, "y": item.y, "z": item.z,
                    "costPoints": item.cost_points, "state": item.state,
                    "failureReason": item.failure_reason
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"history": history})))
    })
}

#[rustfmt::skip]
fn quote_body(policy: &RandomTeleportPolicy, balance: i64, cooldown_remaining: i64) -> serde_json::Value {
    let enabled = matches!(policy.decide(cooldown_remaining), RandomTeleportDecision::Allowed);
    json!({"enabled": enabled, "profileId": policy.profile_id,
        "targetEnvironment": policy.target_environment, "costPoints": policy.cost_points,
        "cooldownSeconds": policy.cooldown_seconds, "cooldownRemainingSeconds": cooldown_remaining,
        "minRadius": policy.min_radius, "maxRadius": policy.max_radius, "maxAttempts": policy.max_attempts,
        "allowedWorlds": policy.allowed_worlds, "worldCandidates": policy.allowed_worlds,
        "balance": balance, "canAfford": balance >= policy.cost_points,
        "confirmationRequired": policy.confirmation_required})
}

fn decision_error(request: &CommandEnvelope, decision: RandomTeleportDecision) -> Option<Response> {
    match decision {
        RandomTeleportDecision::Allowed => None,
        RandomTeleportDecision::Cooldown { remaining_seconds } => Some(api::error(
            request.clone(),
            "rtp.cooldown",
            format!("random teleport cooldown: {remaining_seconds}s"),
            false,
        )),
        RandomTeleportDecision::Disabled(reason) => {
            Some(api::error(request.clone(), "rtp.disabled", reason, false))
        }
    }
}

fn profile_policy(request: &CommandEnvelope) -> Result<RandomTeleportPolicy, String> {
    let profile_id = request
        .body
        .get("profileId")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("overworld");
    RandomTeleportPolicy::profile(profile_id)
        .ok_or_else(|| format!("unknown random teleport profile: {profile_id}"))
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}

fn number(request: &CommandEnvelope, field: &'static str) -> Result<f64, String> {
    request
        .body
        .get(field)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| format!("missing number field: {field}"))
}
