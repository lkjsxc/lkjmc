use lkjmc_core::command::CommandEnvelope;
use serde_json::json;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

pub fn handle(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_client(state, request, |_state, request, client| {
        let id = body_string(&request.body, "id")?;
        if store(lkjmc_store::instance::get(client, &id))?.is_none() {
            return Err(format!("instance not found: {id}"));
        }
        store(lkjmc_store::instance::upsert_observation(
            client,
            &id,
            "process-healthy",
            None,
            true,
            Some("plugin heartbeat"),
        ))?;
        Ok(api::ok(request, json!({"id": id, "heartbeat": true})))
    })
}
