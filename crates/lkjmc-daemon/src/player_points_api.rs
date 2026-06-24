use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn balance(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
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
