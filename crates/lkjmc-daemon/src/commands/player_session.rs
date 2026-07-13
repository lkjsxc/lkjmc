use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn join(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let server = body_string(&request.body, "serverId")?;
        store(lkjmc_store::player_session::join(
            client,
            Uuid::new_v4(),
            player_uuid,
            &name,
            &server,
        ))?;
        Ok(api::ok(
            request,
            json!({"playerUuid": player_uuid.to_string(), "serverId": server}),
        ))
    })
}

pub fn leave(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let server = body_string(&request.body, "serverId")?;
        store(lkjmc_store::player_session::leave(
            client,
            player_uuid,
            &server,
        ))?;
        Ok(api::ok(
            request,
            json!({"playerUuid": player_uuid.to_string(), "serverId": server}),
        ))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
