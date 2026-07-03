use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn request(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let target = body_string(&request.body, "targetServer")?;
        let source = body_string(&request.body, "sourceServer")?;
        let location = request
            .body
            .get("location")
            .cloned()
            .ok_or("missing location")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        store(lkjmc_store::teleport::request(
            client,
            player_uuid,
            &target,
            &source,
            location,
        ))?;
        Ok(api::ok(request, json!({"targetServer": target})))
    })
}

pub fn take(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let server = body_string(&request.body, "serverId")?;
        let Some(location) = store(lkjmc_store::teleport::take(client, player_uuid, &server))?
        else {
            return Ok(api::ok(request, json!({"found": false})));
        };
        Ok(api::ok(
            request,
            json!({"found": true, "location": location}),
        ))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
