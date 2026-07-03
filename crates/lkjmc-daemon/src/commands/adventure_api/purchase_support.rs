use std::fs;
use std::path::Path;

use lkjmc_core::adventure::AdventureDefinition;
use lkjmc_core::command::CommandEnvelope;
use lkjmc_core::id::InstanceId;
use lkjmc_core::temporary::{plan_temporary_instance, CleanupPolicy, TemporaryInstanceRequest};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::adventure_api::participants;
use crate::commands::adventure_api::rows::PurchaseParticipant;
use crate::commands::temporary_api::create_support::runtime_facts;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::{body_string, store};

pub(super) fn temporary_plan(
    state: &AppState,
    client: &mut postgres::Client,
    instance_id: &str,
    world_root: &str,
    max_lifetime_seconds: u32,
    retention_seconds: u32,
) -> Result<lkjmc_core::temporary::TemporaryInstancePlan, String> {
    plan_temporary_instance(
        &TemporaryInstanceRequest {
            instance_id,
            world_root,
            max_lifetime_seconds,
            retention_seconds,
            cleanup_policy: CleanupPolicy::Delete,
        },
        &runtime_facts(state, client)?,
    )
    .map_err(|error| format!("temporary plan failed: {error:?}"))
}

pub(super) fn response(
    definition: &AdventureDefinition,
    session_id: Uuid,
    instance_id: &str,
    ledger: Uuid,
    participants: &[PurchaseParticipant],
) -> Value {
    json!({
        "sessionId": session_id.to_string(),
        "adventureId": definition.id,
        "temporaryInstanceId": instance_id,
        "pointsLedgerId": ledger.to_string(),
        "targetServer": instance_id,
        "state": "ready",
        "participantCount": participants.len(),
        "participants": participants::as_json(participants)
    })
}

pub(super) fn refund_purchase(
    client: &mut postgres::Client,
    session_id: Uuid,
    player_uuid: Uuid,
    cost: i64,
    adventure_id: &str,
    reason: &str,
) -> Result<(), String> {
    let refund = store(lkjmc_store::points::grant_with_correlation(
        client,
        player_uuid,
        cost,
        &format!("{adventure_id}-refund"),
        Some(session_id),
    ))?;
    store(lkjmc_store::temporary::update_session_state(
        client,
        session_id,
        "refunded",
        Some(reason),
        Some(refund),
    ))
}

pub(super) fn prepare_files(
    state: &AppState,
    plan: &lkjmc_core::temporary::TemporaryInstancePlan,
    config: &Value,
) -> Result<(), String> {
    fs::create_dir_all(&plan.world_path).map_err(|error| format!("create world: {error}"))?;
    crate::templates::render_instance(state, &plan.instance_id, "folia", config).map(|_| ())
}

pub(super) fn cleanup_files(state: &AppState, id: &str, world_path: &str) {
    let _ = fs::remove_dir_all(world_path);
    let _ = fs::remove_dir_all(Path::new(&state.data_root()).join(id));
}

pub(super) fn audit_event(
    client: &mut postgres::Client,
    envelope: &CommandEnvelope,
    adventure_id: &str,
    session_id: Uuid,
    result: &str,
) -> Result<(), String> {
    audit(
        client,
        envelope,
        "adventure.purchase",
        adventure_id,
        &session_id.to_string(),
        result,
    )
}

pub(super) fn cost(body: &Value, definition: &AdventureDefinition) -> Result<i64, String> {
    let cost = body
        .get("cost")
        .and_then(Value::as_i64)
        .unwrap_or(definition.price_points);
    if cost <= 0 {
        return Err("cost must be positive".to_string());
    }
    Ok(cost)
}

pub(super) fn validate_party(definition: &AdventureDefinition, count: usize) -> Result<(), String> {
    if count < usize::from(definition.min_party_size)
        || count > usize::from(definition.max_party_size)
    {
        return Err("party size outside adventure bounds".to_string());
    }
    Ok(())
}

pub(super) fn parse_uuid(body: &Value, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(body, field)?).map_err(|error| error.to_string())
}

pub(super) fn seconds(value: u32) -> Result<i32, String> {
    i32::try_from(value).map_err(|error| error.to_string())
}

pub(super) fn instance_id(
    definition: &AdventureDefinition,
    session_id: Uuid,
) -> Result<String, String> {
    let suffix = session_id.simple().to_string();
    let prefix = definition.id.split('-').next().unwrap_or("adv");
    let id = format!("{}-{}", prefix, &suffix[..12]);
    InstanceId::parse(id.clone()).map_err(|error| error.to_string())?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_instance_id_uses_adventure_prefix() {
        let definition = lkjmc_core::adventure::get("nether-fortress-raid");
        let id = definition.and_then(|value| instance_id(value, Uuid::nil()).ok());
        assert_eq!(id.as_deref(), Some("nether-000000000000"));
    }
}
