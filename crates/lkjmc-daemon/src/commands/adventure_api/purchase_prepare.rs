use lkjmc_core::command::CommandEnvelope;
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::adventure_api::participants;
use crate::commands::adventure_api::purchase_support as support;
use crate::commands::adventure_api::rows::{insert_purchase, PurchaseParticipant, PurchaseRows};
use crate::commands::temporary_api::create_support::{ensure_new_world, instance_config};
use crate::commands::temporary_api::request;
use crate::support::instance_helpers::{body_string, store};

pub(super) struct PreparedPurchase {
    pub adventure_id: String,
    pub session_id: Uuid,
    pub instance_id: String,
    pub ledger: Uuid,
    pub participants: Vec<PurchaseParticipant>,
}

pub(super) fn prepare(
    state: &AppState,
    envelope: &CommandEnvelope,
    client: &mut postgres::Client,
) -> Result<PreparedPurchase, String> {
    let adventure_id = body_string(&envelope.body, "adventureId")?;
    let definition = lkjmc_core::adventure::get(&adventure_id)
        .filter(|adventure| adventure.enabled)
        .ok_or_else(|| format!("unknown adventure: {adventure_id}"))?;
    let player_uuid = support::parse_uuid(&envelope.body, "playerUuid")?;
    let player_name = body_string(&envelope.body, "playerName")?;
    let cost = support::cost(&envelope.body, definition)?;
    let participants = participants::collect(
        client,
        player_uuid,
        &player_name,
        participants::include_party(&envelope.body)?,
    )?;
    support::validate_party(definition, participants.len())?;
    let session_id = Uuid::new_v4();
    let instance_id = support::instance_id(definition, session_id)?;
    let world_root = request::optional_string(
        &envelope.body,
        "worldRoot",
        format!("{}/temporary-worlds", state.data_root()),
    );
    let max_lifetime = request::u32_field(
        &envelope.body,
        "maxLifetimeSeconds",
        definition.max_lifetime_seconds,
    )?;
    let retention = request::u32_field(
        &envelope.body,
        "retentionSeconds",
        definition.retention_seconds,
    )?;
    support::seconds(max_lifetime)?;
    support::seconds(retention)?;
    let plan = support::temporary_plan(
        state,
        client,
        &instance_id,
        &world_root,
        max_lifetime,
        retention,
    )?;
    ensure_new_world(&plan.world_path)?;
    let jar = store(lkjmc_store::jar::latest_matching(
        client,
        definition.jar_kind,
    ))?
    .ok_or_else(|| format!("{} jar asset not found", definition.jar_kind))?;
    let config = instance_config(state, &plan, &jar.id.to_string())?;
    support::prepare_files(state, &plan, &config)?;
    let session_id_text = session_id.to_string();
    let rows = PurchaseRows {
        session_id,
        session_id_text: &session_id_text,
        player_uuid,
        player_name: &player_name,
        adventure_id: definition.id,
        cost,
        plan: &plan,
        config: &config,
        jar_id: jar.id,
        participants: &participants,
    };
    let ledger = match insert_purchase(client, rows) {
        Ok(ledger) => ledger,
        Err(error) => {
            support::cleanup_files(state, &plan.instance_id, &plan.world_path);
            return Err(error);
        }
    };
    Ok(PreparedPurchase {
        adventure_id,
        session_id,
        instance_id,
        ledger,
        participants,
    })
}
