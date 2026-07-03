use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::claim_create::{int, operator, uuid};
use crate::instance_helpers::{body_string, store, with_connection};

pub fn trust(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let owner_uuid = uuid(&request, "ownerUuid")?;
        let trusted_uuid = uuid(&request, "trustedUuid")?;
        let trusted_name = body_string(&request.body, "trustedName")?;
        let claim = target_claim(client, &request, owner_uuid)?;
        store(lkjmc_store::claims::trust_player(
            client,
            claim,
            trusted_uuid,
            &trusted_name,
        ))?;
        crate::audit_helpers::audit(
            client,
            &request,
            "claim.trust",
            "claim",
            &claim.to_string(),
            "succeeded",
        )?;
        Ok(api::ok(
            request,
            json!({"claimId": claim.to_string(), "trusted": true}),
        ))
    })
}

pub fn untrust(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let owner_uuid = uuid(&request, "ownerUuid")?;
        let trusted_uuid = uuid(&request, "trustedUuid")?;
        let claim = target_claim(client, &request, owner_uuid)?;
        let removed = store(lkjmc_store::claims::untrust_player(
            client,
            claim,
            trusted_uuid,
        ))?;
        crate::audit_helpers::audit(
            client,
            &request,
            "claim.untrust",
            "claim",
            &claim.to_string(),
            "succeeded",
        )?;
        Ok(api::ok(
            request,
            json!({"claimId": claim.to_string(), "removed": removed}),
        ))
    })
}

fn target_claim(
    client: &mut postgres::Client,
    request: &CommandEnvelope,
    owner_uuid: Uuid,
) -> Result<Uuid, String> {
    if let Some(name) = request.body.get("name").and_then(serde_json::Value::as_str) {
        let claim = store(lkjmc_store::claims::active_claim_by_owner_name(
            client, owner_uuid, name,
        ))?
        .ok_or_else(|| "claim not found".to_string())?;
        return Ok(claim.id);
    }
    let instance_id = body_string(&request.body, "instanceId")?;
    let world_name = body_string(&request.body, "worldName")?;
    let chunk_x = int(request, "chunkX")?;
    let chunk_z = int(request, "chunkZ")?;
    let claim = store(lkjmc_store::claims::lookup_claim_by_chunk(
        client,
        &instance_id,
        &world_name,
        chunk_x,
        chunk_z,
    ))?
    .ok_or_else(|| "claim not found".to_string())?;
    if claim.owner_uuid != owner_uuid && !operator(request) {
        return Err("not claim owner".to_string());
    }
    Ok(claim.claim_id)
}
