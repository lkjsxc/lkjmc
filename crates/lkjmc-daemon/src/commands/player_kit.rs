use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let kits = store(lkjmc_store::kits::list(client))?
            .into_iter()
            .map(|kit| {
                json!({
                    "id": kit.id,
                    "titleKey": kit.title_key,
                    "rewardPoints": kit.reward_points,
                    "cooldownHours": kit.cooldown_hours
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"kits": kits})))
    })
}

pub fn claim(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let kit_id = body_string(&request.body, "kitId")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        let kit = store(lkjmc_store::kits::get(client, &kit_id))?
            .ok_or_else(|| format!("kit not found: {kit_id}"))?;
        let claimed = store(lkjmc_store::kits::claim(client, player_uuid, &kit))?;
        if claimed {
            store(lkjmc_store::points::grant(
                client,
                player_uuid,
                kit.reward_points,
                &format!("kit.claim:{}", kit.id),
            ))?;
        }
        Ok(api::ok(
            request,
            json!({
                "kitId": kit.id,
                "claimed": claimed,
                "rewardPoints": kit.reward_points
            }),
        ))
    })
}

pub fn upsert(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let kit_id = body_string(&request.body, "kitId")?;
        let title_key = body_string(&request.body, "titleKey")?;
        let reward_points = number(&request, "rewardPoints")?;
        let cooldown_hours =
            i32::try_from(number(&request, "cooldownHours")?).map_err(|error| error.to_string())?;
        store(lkjmc_store::kits::upsert(
            client,
            &kit_id,
            &title_key,
            reward_points,
            cooldown_hours,
        ))?;
        Ok(api::ok(request, json!({"kitId": kit_id})))
    })
}

fn number(request: &CommandEnvelope, field: &'static str) -> Result<i64, String> {
    request
        .body
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| format!("missing number field: {field}"))
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
