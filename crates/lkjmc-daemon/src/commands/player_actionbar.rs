use lkjmc_core::command::CommandEnvelope;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn snapshot(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let key = parse_uuid(&request, "playerUuid")?.to_string();
        let result = store(lkjmc_store::sync::snapshot(client, "settings", &key))?;
        let body = match result {
            lkjmc_store::sync::SnapshotResult::Available(value) => json!({
                "result": "snapshot", "domain": value.domain, "key": value.key,
                "revision": value.revision, "generatedAt": value.generated_at,
                "payload": value.payload
            }),
            lkjmc_store::sync::SnapshotResult::Unavailable { reason } => json!({
                "result": "unavailable", "domain": "settings", "key": key,
                "reason": reason
            }),
        };
        Ok(api::ok(request, body))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
