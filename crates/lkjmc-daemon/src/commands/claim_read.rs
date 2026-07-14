use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;

use crate::app::AppState;
use crate::commands::claim_create::uuid;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

pub fn list(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let owner_uuid = uuid(&request, "ownerUuid")?;
        let claims = store(lkjmc_store::claims::list_claims_for_owner(
            client, owner_uuid,
        ))?
        .into_iter()
        .map(|claim| {
            json!({
                "id": claim.id.to_string(), "ownerUuid": claim.owner_uuid.to_string(),
                "ownerName": claim.owner_name, "name": claim.name,
                "chunkCount": claim.chunk_count
            })
        })
        .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"claims": claims})))
    })
}

pub fn snapshot(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let key = body_string(&request.body, "instanceId")?;
        let result = store(lkjmc_store::sync::snapshot(client, "claims", &key))?;
        Ok(api::ok(request, sync_body("claims", &key, result)))
    })
}

fn sync_body(
    domain: &str,
    key: &str,
    result: lkjmc_store::sync::SnapshotResult,
) -> serde_json::Value {
    match result {
        lkjmc_store::sync::SnapshotResult::Available(value) => json!({
            "result": "snapshot", "domain": value.domain, "key": value.key,
            "revision": value.revision, "generatedAt": value.generated_at,
            "payload": value.payload
        }),
        lkjmc_store::sync::SnapshotResult::Unavailable { reason } => json!({
            "result": "unavailable", "domain": domain, "key": key, "reason": reason
        }),
    }
}
