use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let rows = store(lkjmc_store::achievement::list_progress(client, player_uuid))?;
        let achievements = rows
            .into_iter()
            .map(|row| {
                json!({
                    "id": row.id,
                    "titleKey": row.title_key,
                    "descriptionKey": row.description_key,
                    "category": row.category,
                    "iconMaterial": row.icon_material,
                    "current": row.current,
                    "required": row.required,
                    "claimed": row.claimed,
                    "rewardClaimed": row.reward_claimed
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"achievements": achievements})))
    })
}

pub fn grant(state: &AppState, request: CommandEnvelope) -> Response {
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
        let achievement_id = body_string(&request.body, "achievementId")?;
        let title_key = body_string(&request.body, "titleKey")?;
        store(lkjmc_store::achievement::grant(
            client,
            player_uuid,
            &achievement_id,
            &title_key,
        ))?;
        Ok(api::ok(request, json!({"achievementId": achievement_id})))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
