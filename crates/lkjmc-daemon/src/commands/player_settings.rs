use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn get(state: &AppState, request: CommandEnvelope) -> Response {
    let player_uuid = match parse_uuid(&request.body, "playerUuid") {
        Ok(value) => value,
        Err(error) => return invalid(request, error),
    };
    let mut client = match state.database_connection() {
        Ok(client) => client,
        Err(_) => return database_unavailable(request),
    };
    let settings = match lkjmc_store::player_settings::current(&mut client, player_uuid) {
        Ok(value) => value,
        Err(error) => return api::database_error(request, error),
    };
    let body = match settings {
        Some(settings) => json!({
            "playerUuid": player_uuid.to_string(),
            "language": settings.language,
            "hudEnabled": settings.hud_enabled,
            "menuEnabled": settings.menu_enabled
        }),
        None => json!({
            "playerUuid": player_uuid.to_string(),
            "language": "en",
            "hudEnabled": false,
            "menuEnabled": true
        }),
    };
    api::ok(request, body)
}

pub fn set_language(state: &AppState, request: CommandEnvelope) -> Response {
    let player_uuid = match parse_uuid(&request.body, "playerUuid") {
        Ok(value) => value,
        Err(error) => return invalid(request, error),
    };
    let name = match body_string(&request.body, "name") {
        Ok(value) => value,
        Err(error) => return invalid(request, error),
    };
    let language = match body_string(&request.body, "language") {
        Ok(value) if matches!(value.as_str(), "en" | "ja") => value,
        Ok(_) => return invalid(request, "language must be en or ja".to_string()),
        Err(error) => return invalid(request, error),
    };
    let mut client = match state.database_connection() {
        Ok(client) => client,
        Err(_) => return database_unavailable(request),
    };
    match lkjmc_store::player_settings::set_language_for_identity(
        &mut client,
        player_uuid,
        &name,
        &language,
    ) {
        Ok(()) => api::ok(
            request,
            json!({"playerUuid": player_uuid.to_string(), "language": language}),
        ),
        Err(error) => api::database_error(request, error),
    }
}

pub fn set_hud(state: &AppState, request: CommandEnvelope) -> Response {
    let player_uuid = match parse_uuid(&request.body, "playerUuid") {
        Ok(value) => value,
        Err(error) => return invalid(request, error),
    };
    let name = match body_string(&request.body, "name") {
        Ok(value) => value,
        Err(error) => return invalid(request, error),
    };
    let Some(enabled) = request
        .body
        .get("enabled")
        .and_then(serde_json::Value::as_bool)
    else {
        return invalid(request, "missing boolean field: enabled".to_string());
    };
    let mut client = match state.database_connection() {
        Ok(client) => client,
        Err(_) => return database_unavailable(request),
    };
    match lkjmc_store::player_settings::set_hud_for_identity(
        &mut client,
        player_uuid,
        &name,
        enabled,
    ) {
        Ok(()) => api::ok(
            request,
            json!({"playerUuid": player_uuid.to_string(), "hudEnabled": enabled}),
        ),
        Err(error) => api::database_error(request, error),
    }
}

pub fn toggle(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request.body, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let setting = body_string(&request.body, "settingKey")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        let body = match setting.as_str() {
            "hud" => json!({
                "playerUuid": player_uuid.to_string(),
                "hudEnabled": store(lkjmc_store::player_settings::toggle_hud(client, player_uuid))?
            }),
            "menu-token" => json!({
                "playerUuid": player_uuid.to_string(),
                "menuEnabled": store(lkjmc_store::player_settings::toggle_menu_enabled(client, player_uuid))?
            }),
            _ => return Err("settingKey must be hud or menu-token".to_string()),
        };
        Ok(api::ok(request, body))
    })
}

fn invalid(request: CommandEnvelope, error: String) -> Response {
    api::error(request, "request.invalid_body", error, false)
}

fn database_unavailable(request: CommandEnvelope) -> Response {
    api::error(request, "database.error", "database checkout failed", true)
}

fn parse_uuid(body: &serde_json::Value, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(body, field)?).map_err(|error| error.to_string())
}
