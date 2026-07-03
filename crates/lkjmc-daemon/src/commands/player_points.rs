use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn balance(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = Uuid::parse_str(&body_string(&request.body, "playerUuid")?)
            .map_err(|error| error.to_string())?;
        let name = body_string(&request.body, "name")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        store(lkjmc_store::points::ensure_account(client, player_uuid))?;
        let balance = store(lkjmc_store::points::balance(client, player_uuid))?;
        Ok(api::ok(
            request,
            json!({"playerUuid": player_uuid.to_string(), "balance": balance}),
        ))
    })
}

pub fn top(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let limit = request
            .body
            .get("limit")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(10);
        let players = store(lkjmc_store::points::top(client, limit))?
            .into_iter()
            .map(|item| {
                json!({
                    "playerUuid": item.player_uuid.to_string(),
                    "name": item.name,
                    "balance": item.balance
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"players": players})))
    })
}
