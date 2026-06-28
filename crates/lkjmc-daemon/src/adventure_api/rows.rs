use serde_json::{json, Value};
use uuid::Uuid;

use crate::instance_helpers::store;

pub(super) struct PurchaseParticipant {
    pub player_uuid: Uuid,
    pub player_name: String,
    pub role: String,
}

pub(super) struct PurchaseRows<'a> {
    pub session_id: Uuid,
    pub session_id_text: &'a str,
    pub player_uuid: Uuid,
    pub player_name: &'a str,
    pub cost: i64,
    pub plan: &'a lkjmc_core::temporary::TemporaryInstancePlan,
    pub config: &'a Value,
    pub jar_id: Uuid,
    pub participants: &'a [PurchaseParticipant],
}

pub(super) fn insert_purchase(
    client: &mut postgres::Client,
    rows: PurchaseRows<'_>,
) -> Result<Uuid, String> {
    let mut tx = client.transaction().map_err(|error| error.to_string())?;
    store(lkjmc_store::player::insert_identity(
        &mut tx,
        rows.player_uuid,
        rows.player_name,
    ))?;
    let ledger = store(lkjmc_store::points::spend_with_correlation(
        &mut tx,
        rows.player_uuid,
        rows.cost,
        "end-expedition",
        Some(rows.session_id),
    ))?
    .ok_or_else(|| "insufficient points".to_string())?;
    store(lkjmc_store::instance::insert(
        &mut tx,
        &rows.plan.instance_id,
        None,
        "folia",
        "stopped",
        rows.config,
    ))?;
    store(lkjmc_store::instance::reserve_port(
        &mut tx,
        &rows.plan.instance_id,
        i32::from(rows.plan.server_port),
        "server",
    ))?;
    store(lkjmc_store::instance::set_jar_asset(
        &mut tx,
        &rows.plan.instance_id,
        rows.jar_id,
    ))?;
    store(lkjmc_store::temporary::insert_instance(
        &mut tx,
        temporary_row(&rows),
    ))?;
    store(lkjmc_store::temporary::insert_session(
        &mut tx,
        session_row(&rows, ledger),
    ))?;
    for participant in rows.participants {
        store(lkjmc_store::temporary::add_participant(
            &mut tx,
            participant_row(&rows, participant),
        ))?;
    }
    tx.commit().map_err(|error| error.to_string())?;
    Ok(ledger)
}

fn temporary_row<'a>(
    rows: &'a PurchaseRows<'_>,
) -> lkjmc_store::temporary::NewTemporaryInstance<'a> {
    lkjmc_store::temporary::NewTemporaryInstance {
        instance_id: &rows.plan.instance_id,
        owner_kind: "adventure",
        owner_id: rows.session_id_text,
        visibility: &rows.plan.visibility,
        world_path: &rows.plan.world_path,
        server_port: i32::from(rows.plan.server_port),
        max_lifetime_seconds: rows.plan.max_lifetime_seconds as i32,
        retention_seconds: rows.plan.retention_seconds as i32,
        cleanup_policy: rows.plan.cleanup_policy.as_str(),
        lifecycle_state: "created",
        start_deadline_seconds: 120,
        metadata: json!({"adventure":"end-expedition"}),
    }
}

fn session_row<'a>(
    rows: &'a PurchaseRows<'_>,
    ledger: Uuid,
) -> lkjmc_store::temporary::NewAdventureSession<'a> {
    lkjmc_store::temporary::NewAdventureSession {
        id: rows.session_id,
        adventure_kind: "end-expedition",
        buyer_uuid: rows.player_uuid,
        buyer_name: rows.player_name,
        temporary_instance_id: &rows.plan.instance_id,
        points_cost: rows.cost,
        points_ledger_id: Some(ledger),
        state: "pending",
        start_deadline_seconds: 120,
        stop_deadline_seconds: rows.plan.max_lifetime_seconds as i32,
        metadata: json!({}),
    }
}

fn participant_row<'a>(
    rows: &'a PurchaseRows<'_>,
    participant: &'a PurchaseParticipant,
) -> lkjmc_store::temporary::NewAdventureParticipant<'a> {
    lkjmc_store::temporary::NewAdventureParticipant {
        session_id: rows.session_id,
        player_uuid: participant.player_uuid,
        player_name: &participant.player_name,
        role: &participant.role,
        state: "queued",
        metadata: json!({}),
    }
}
