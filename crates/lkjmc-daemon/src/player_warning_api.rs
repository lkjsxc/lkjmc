use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn create(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
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
        store(lkjmc_store::warnings::create(
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

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let limit = request
            .body
            .get("limit")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(20)
            .clamp(1, 100);
        let warnings = store(lkjmc_store::warnings::list(client, player_uuid, limit))?
            .into_iter()
            .map(|warning| {
                json!({
                    "id": warning.id.to_string(),
                    "playerUuid": warning.player_uuid.to_string(),
                    "playerName": warning.player_name,
                    "actorName": warning.actor_name,
                    "reason": warning.reason
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"warnings": warnings})))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
