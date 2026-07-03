use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

pub fn handle(state: &AppState, request: CommandEnvelope) -> lkjmc_core::command::CommandResponse {
    with_connection(state, request, |_state, request, client| {
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
        store(lkjmc_store::instance_presence::upsert_heartbeat(
            client,
            lkjmc_store::instance_presence::PresenceHeartbeat {
                instance_id: &id,
                player_count: optional_i32(&request.body, "playerCount")?,
                max_players: optional_i32(&request.body, "maxPlayers")?,
                ready: request
                    .body
                    .get("ready")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                implementation: request.body.get("implementation").and_then(Value::as_str),
            },
        ))?;
        Ok(api::ok(request, json!({"id": id, "heartbeat": true})))
    })
}

fn optional_i32(body: &Value, key: &'static str) -> Result<Option<i32>, String> {
    body.get(key)
        .and_then(Value::as_i64)
        .map(i32::try_from)
        .transpose()
        .map_err(|error| format!("invalid {key}: {error}"))
}
