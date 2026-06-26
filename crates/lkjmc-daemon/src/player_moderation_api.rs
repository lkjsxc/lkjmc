use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn ban(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let player_name = body_string(&request.body, "playerName")?;
        let actor_name = body_string(&request.body, "actorName")?;
        let reason = body_string(&request.body, "reason")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &player_name,
        ))?;
        let id = Uuid::new_v4();
        store(lkjmc_store::moderation::ban(
            client,
            id,
            player_uuid,
            &player_name,
            &actor_name,
            &reason,
        ))?;
        Ok(api::ok(request, json!({"id": id.to_string()})))
    })
}

pub fn mute(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let player_name = body_string(&request.body, "playerName")?;
        let actor_name = body_string(&request.body, "actorName")?;
        let reason = body_string(&request.body, "reason")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &player_name,
        ))?;
        let id = Uuid::new_v4();
        store(lkjmc_store::moderation::mute(
            client,
            id,
            player_uuid,
            &player_name,
            &actor_name,
            &reason,
        ))?;
        Ok(api::ok(request, json!({"id": id.to_string()})))
    })
}

pub fn unban(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_name = body_string(&request.body, "playerName")?;
        let revoked = store(lkjmc_store::moderation::revoke_ban(client, &player_name))?;
        Ok(api::ok(
            request,
            json!({"playerName": player_name, "revoked": revoked}),
        ))
    })
}

pub fn unmute(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_name = body_string(&request.body, "playerName")?;
        let revoked = store(lkjmc_store::moderation::revoke_mute(client, &player_name))?;
        Ok(api::ok(
            request,
            json!({"playerName": player_name, "revoked": revoked}),
        ))
    })
}

pub fn status(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        if let Some(name) = request
            .body
            .get("playerName")
            .and_then(serde_json::Value::as_str)
        {
            store(lkjmc_store::player::insert_identity(
                client,
                player_uuid,
                name,
            ))?;
        }
        let ban = store(lkjmc_store::moderation::active_ban(client, player_uuid))?;
        let mute = store(lkjmc_store::moderation::active_mute(client, player_uuid))?;
        Ok(api::ok(
            request,
            json!({
                "banned": ban.is_some(),
                "muted": mute.is_some(),
                "reason": ban.map(|item| item.reason).unwrap_or_default(),
                "muteReason": mute.map(|item| item.reason).unwrap_or_default()
            }),
        ))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
