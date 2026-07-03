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
                "id": claim.id.to_string(),
                "ownerUuid": claim.owner_uuid.to_string(),
                "ownerName": claim.owner_name,
                "name": claim.name,
                "chunkCount": claim.chunk_count
            })
        })
        .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"claims": claims})))
    })
}

pub fn snapshot(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let instance_id = body_string(&request.body, "instanceId")?;
        let chunks = store(lkjmc_store::claims::snapshot_claim_chunks(
            client,
            &instance_id,
        ))?
        .into_iter()
        .map(|chunk| {
            let trusts = chunk.trusts.into_iter().map(|trust| {
                    json!({"uuid": trust.trusted_uuid.to_string(), "name": trust.trusted_name})
                }).collect::<Vec<_>>();
            json!({
                "claimId": chunk.claim_id.to_string(),
                "ownerUuid": chunk.owner_uuid.to_string(),
                "ownerName": chunk.owner_name,
                "name": chunk.name,
                "instanceId": chunk.instance_id,
                "worldName": chunk.world_name,
                "chunkX": chunk.chunk_x,
                "chunkZ": chunk.chunk_z,
                "trusts": trusts
            })
        })
        .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"chunks": chunks})))
    })
}
