use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn set(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let home = body_string(&request.body, "home")?;
        let server_id = body_string(&request.body, "serverId")?;
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
        store(lkjmc_store::homes::upsert(
            client,
            Uuid::new_v4(),
            player_uuid,
            &home,
            &server_id,
            location,
        ))?;
        store(lkjmc_store::achievement::apply_event(
            client,
            player_uuid,
            "home-set",
            1,
            None,
        ))?;
        Ok(api::ok(
            request,
            json!({"home": home, "serverId": server_id}),
        ))
    })
}

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let homes = store(lkjmc_store::homes::list(client, player_uuid))?
            .into_iter()
            .map(|home| json!({"home": home.name, "serverId": home.server_id, "location": home.location}))
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"homes": homes})))
    })
}

pub fn get(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let home = body_string(&request.body, "home")?;
        let Some(record) = store(lkjmc_store::homes::get(client, player_uuid, &home))? else {
            return Ok(api::ok(request, json!({"found": false})));
        };
        Ok(api::ok(
            request,
            json!({"found": true, "home": record.name, "serverId": record.server_id, "location": record.location}),
        ))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
