use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn create(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let player_name = body_string(&request.body, "playerName")?;
        let actor_name = body_string(&request.body, "actorName")?;
        let body = body_string(&request.body, "body")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &player_name,
        ))?;
        let id = Uuid::new_v4();
        store(lkjmc_store::notes::create(
            client,
            id,
            player_uuid,
            &player_name,
            &actor_name,
            &body,
        ))?;
        Ok(api::ok(request, json!({"noteId": id.to_string()})))
    })
}

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let limit = request
            .body
            .get("limit")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(20);
        let notes = store(lkjmc_store::notes::list(client, player_uuid, limit))?
            .into_iter()
            .map(|note| {
                json!({
                    "id": note.id.to_string(),
                    "playerUuid": note.player_uuid.to_string(),
                    "playerName": note.player_name,
                    "actorName": note.actor_name,
                    "body": note.body
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"notes": notes})))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
