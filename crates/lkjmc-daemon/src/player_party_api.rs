use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn create(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let player_name = body_string(&request.body, "playerName")?;
        let party_name = body_string(&request.body, "partyName")?;
        if store(lkjmc_store::party::current(client, player_uuid))?.is_some() {
            return Err("player already has a party".to_string());
        }
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &player_name,
        ))?;
        let party_id = Uuid::new_v4();
        store(lkjmc_store::party::create(
            client,
            party_id,
            player_uuid,
            &party_name,
        ))?;
        Ok(api::ok(
            request,
            json!({"partyId": party_id.to_string(), "name": party_name}),
        ))
    })
}

pub fn info(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let Some(party) = store(lkjmc_store::party::current(client, player_uuid))? else {
            return Ok(api::ok(request, json!({"found": false})));
        };
        Ok(api::ok(
            request,
            json!({"found": true, "partyId": party.id.to_string(), "name": party.name, "role": party.role}),
        ))
    })
}

pub fn leave(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let removed = store(lkjmc_store::party::leave(client, player_uuid))?;
        let _ = store(lkjmc_store::party::delete_empty(client))?;
        Ok(api::ok(request, json!({"removed": removed})))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
