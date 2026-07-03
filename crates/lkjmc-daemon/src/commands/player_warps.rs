use lkjmc_core::command::CommandEnvelope;
use serde_json::json;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn set(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let name = body_string(&request.body, "warp")?;
        let server_id = body_string(&request.body, "serverId")?;
        let location = request
            .body
            .get("location")
            .cloned()
            .ok_or("missing location")?;
        store(lkjmc_store::warps::upsert(
            client, &name, &server_id, location,
        ))?;
        Ok(api::ok(
            request,
            json!({"warp": name, "serverId": server_id}),
        ))
    })
}

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let warps = store(lkjmc_store::warps::list(client))?
            .into_iter()
            .map(|warp| json!({"warp": warp.name, "serverId": warp.server_id, "location": warp.location}))
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"warps": warps})))
    })
}

pub fn get(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let name = body_string(&request.body, "warp")?;
        let Some(record) = store(lkjmc_store::warps::get(client, &name))? else {
            return Ok(api::ok(request, json!({"found": false})));
        };
        Ok(api::ok(
            request,
            json!({"found": true, "warp": record.name, "serverId": record.server_id, "location": record.location}),
        ))
    })
}
