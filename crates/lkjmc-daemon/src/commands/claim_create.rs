use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection, with_transaction};

pub fn create(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_transaction(state, request, |_state, request, tx| {
        let claim_id = Uuid::new_v4();
        let owner_uuid = uuid(&request, "ownerUuid")?;
        let owner_name = body_string(&request.body, "ownerName")?;
        let name = body_string(&request.body, "name")?;
        let instance_id = body_string(&request.body, "instanceId")?;
        let world_name = body_string(&request.body, "worldName")?;
        let claim = lkjmc_store::claims::NewClaim {
            id: claim_id,
            owner_uuid,
            owner_name: &owner_name,
            name: &name,
            instance_id: &instance_id,
            world_name: &world_name,
            chunk_x: int(&request, "chunkX")?,
            chunk_z: int(&request, "chunkZ")?,
        };
        store(lkjmc_store::claims::create_claim_in(tx, claim))?;
        store(lkjmc_store::achievement::apply_event_for_player(
            tx,
            owner_uuid,
            Some(&owner_name),
            "claim-created",
            1,
            Some(claim_id),
        ))?;
        crate::support::audit_helpers::audit(
            tx,
            &request,
            "claim.create",
            "claim",
            &claim_id.to_string(),
            "succeeded",
        )?;
        Ok(api::ok(request, json!({"claimId": claim_id.to_string()})))
    })
}

pub fn delete(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    with_connection(state, request, |_state, request, client| {
        let claim_id = if operator(&request) {
            request
                .body
                .get("claimId")
                .and_then(|value| value.as_str())
                .map(parse_uuid)
                .transpose()?
        } else {
            None
        };
        let claim_id = match claim_id {
            Some(value) => value,
            None => {
                let owner_uuid = uuid(&request, "ownerUuid")?;
                let name = body_string(&request.body, "name")?;
                store(lkjmc_store::claims::active_claim_by_owner_name(
                    client, owner_uuid, &name,
                ))?
                .ok_or_else(|| "claim not found".to_string())?
                .id
            }
        };
        let deleted = store(lkjmc_store::claims::delete_claim(client, claim_id))?;
        crate::support::audit_helpers::audit(
            client,
            &request,
            "claim.delete",
            "claim",
            &claim_id.to_string(),
            "succeeded",
        )?;
        Ok(api::ok(
            request,
            json!({"claimId": claim_id.to_string(), "deleted": deleted}),
        ))
    })
}

pub(crate) fn uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    parse_uuid(&body_string(&request.body, field)?)
}

pub(crate) fn parse_uuid(value: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|error| error.to_string())
}

pub(crate) fn int(request: &CommandEnvelope, field: &'static str) -> Result<i32, String> {
    request
        .body
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| format!("missing integer field: {field}"))
}

pub(crate) fn operator(request: &CommandEnvelope) -> bool {
    request
        .body
        .get("operator")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}
