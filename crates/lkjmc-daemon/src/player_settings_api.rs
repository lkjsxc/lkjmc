use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn set_language(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let language = body_string(&request.body, "language")?;
        if !matches!(language.as_str(), "en" | "ja") {
            return Err("language must be en or ja".to_string());
        }
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        store(lkjmc_store::player_settings::set_language(
            client,
            player_uuid,
            &language,
        ))?;
        Ok(api::ok(
            request,
            json!({"playerUuid": player_uuid.to_string(), "language": language}),
        ))
    })
}

pub fn set_hud(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let enabled = request
            .body
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| "missing boolean field: enabled".to_string())?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        store(lkjmc_store::player_settings::set_hud(
            client,
            player_uuid,
            enabled,
        ))?;
        Ok(api::ok(
            request,
            json!({"playerUuid": player_uuid.to_string(), "hudEnabled": enabled}),
        ))
    })
}

fn parse_uuid(body: &serde_json::Value, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(body, field)?).map_err(|error| error.to_string())
}
