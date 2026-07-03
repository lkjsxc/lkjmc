use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let links = store(lkjmc_store::votes::list(client))?
            .into_iter()
            .map(|link| {
                json!({
                    "id": link.id,
                    "titleKey": link.title_key,
                    "url": link.url,
                    "sortOrder": link.sort_order
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"links": links})))
    })
}

pub fn upsert(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let id = body_string(&request.body, "id")?;
        let title_key = body_string(&request.body, "titleKey")?;
        let url = body_string(&request.body, "url")?;
        let sort_order = number_or(&request, "sortOrder", 0);
        let sort_order = i32::try_from(sort_order).map_err(|error| error.to_string())?;
        store(lkjmc_store::votes::upsert(
            client, &id, &title_key, &url, sort_order,
        ))?;
        Ok(api::ok(request, json!({"id": id})))
    })
}

pub fn reward(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let player_name = body_string(&request.body, "playerName")?;
        let link_id = body_string(&request.body, "linkId")?;
        let points = number_or(&request, "points", 0);
        if points <= 0 {
            return Err("points must be positive".to_string());
        }
        let source = request
            .body
            .get("source")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("cli");
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &player_name,
        ))?;
        let id = store(lkjmc_store::votes::reward(
            client,
            player_uuid,
            &player_name,
            &link_id,
            points,
            source,
        ))?;
        Ok(api::ok(request, json!({"rewardId": id.to_string()})))
    })
}

fn number_or(request: &CommandEnvelope, field: &'static str, default: i64) -> i64 {
    request
        .body
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(default)
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
