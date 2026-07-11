mod cancel;
mod participants;
mod purchase;
mod purchase_replay;
mod purchase_support;
mod return_to_hub;
mod rows;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

pub(crate) fn replay_purchase(
    client: &mut postgres::Client,
    player: Uuid,
    correlation: Uuid,
) -> Result<Option<Value>, String> {
    purchase_replay::by_correlation(client, player, correlation)
}

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "adventure.catalog.list" => catalog(request),
        "adventure.purchase" => purchase::purchase(state, request),
        "adventure.return" => return_to_hub::generic(state, request),
        "adventure.session.get" => session_get(state, request),
        "adventure.session.list" => session_list(state, request),
        "adventure.session.cancel" => cancel::handle(state, request),
        "adventure.end.purchase" => purchase::end(state, request),
        "adventure.end.return" => return_to_hub::end(state, request),
        _ => api::error(
            request,
            "command.unknown",
            "unknown adventure command",
            false,
        ),
    }
}

fn catalog(request: CommandEnvelope) -> CommandResponse {
    let adventures = lkjmc_core::adventure::DEFAULT_ADVENTURES
        .iter()
        .map(|item| {
            json!({
                "id": item.id,
                "titleKey": item.title_key,
                "loreKey": item.lore_key,
                "category": item.category,
                "iconMaterial": item.icon_material,
                "pricePoints": item.price_points,
                "minPartySize": item.min_party_size,
                "maxPartySize": item.max_party_size,
                "maxLifetimeSeconds": item.max_lifetime_seconds,
                "retentionSeconds": item.retention_seconds,
                "runtimeKind": item.runtime_kind,
                "jarKind": item.jar_kind,
                "worldProfile": item.world_profile,
                "cleanupPolicy": item.cleanup_policy,
                "permission": item.permission,
                "enabled": item.enabled
            })
        })
        .collect::<Vec<_>>();
    api::ok(request, json!({"adventures": adventures}))
}

fn session_get(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let session = store(lkjmc_store::temporary::active_session_for_player(
            client,
            player_uuid,
        ))?;
        Ok(api::ok(
            request,
            json!({"session": session.map(session_json)}),
        ))
    })
}

fn session_list(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let limit = request
            .body
            .get("lines")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(50);
        let sessions = store(lkjmc_store::temporary::list_sessions(client, limit))?
            .into_iter()
            .map(session_json)
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"sessions": sessions})))
    })
}

fn session_json(row: lkjmc_store::temporary::AdventureSessionSummary) -> serde_json::Value {
    json!({
        "sessionId": row.id.to_string(),
        "adventureId": row.adventure_kind,
        "buyerUuid": row.buyer_uuid.to_string(),
        "temporaryInstanceId": row.temporary_instance_id,
        "state": row.state,
        "pointsCost": row.points_cost
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
