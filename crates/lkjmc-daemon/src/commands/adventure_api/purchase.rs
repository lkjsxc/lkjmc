use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::Value;
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::adventure_api::participants;
use crate::commands::adventure_api::purchase_support as support;
use crate::commands::adventure_api::rows::{insert_purchase, PurchaseRows};
use crate::commands::temporary_api::create_support::{
    ensure_new_world, instance_config, read_forwarding_secret,
};
use crate::commands::temporary_api::lifecycle::start_ready;
use crate::commands::temporary_api::request;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

pub fn end(state: &AppState, mut envelope: CommandEnvelope) -> CommandResponse {
    envelope.body["adventureId"] = Value::String("end-expedition".to_string());
    purchase(state, envelope)
}

pub fn purchase(state: &AppState, envelope: CommandEnvelope) -> CommandResponse {
    with_connection(state, envelope, |state, envelope, client| {
        request::require_eula(&envelope.body)?;
        let adventure_id = body_string(&envelope.body, "adventureId")?;
        let definition = lkjmc_core::adventure::get(&adventure_id)
            .filter(|adventure| adventure.enabled)
            .ok_or_else(|| format!("unknown adventure: {adventure_id}"))?;
        let player_uuid = support::parse_uuid(&envelope.body, "playerUuid")?;
        let player_name = body_string(&envelope.body, "playerName")?;
        let cost = support::cost(&envelope.body, definition)?;
        let session_id = support::correlation(&envelope.body)?.unwrap_or_else(Uuid::new_v4);
        if let Some(session) = store(lkjmc_store::temporary::get_session(client, session_id))? {
            if session.buyer_uuid != player_uuid
                || session.adventure_kind != definition.id
                || session.points_cost != cost
            {
                return Err("adventure correlation does not match the settled session".to_string());
            }
            return Ok(api::ok(
                envelope,
                support::replay_response(definition, &session),
            ));
        }
        let include_party = participants::include_party(&envelope.body)?;
        let participants = participants::collect(client, player_uuid, &player_name, include_party)?;
        support::validate_party(definition, participants.len())?;
        let instance_id = support::instance_id(definition, session_id)?;
        let world_root = request::optional_string(
            &envelope.body,
            "worldRoot",
            format!("{}/temporary-worlds", state.data_root()),
        );
        let max_lifetime_seconds = request::u32_field(
            &envelope.body,
            "maxLifetimeSeconds",
            definition.max_lifetime_seconds,
        )?;
        let retention_seconds = request::u32_field(
            &envelope.body,
            "retentionSeconds",
            definition.retention_seconds,
        )?;
        support::seconds(max_lifetime_seconds)?;
        support::seconds(retention_seconds)?;
        let plan = support::temporary_plan(
            state,
            client,
            &instance_id,
            &world_root,
            max_lifetime_seconds,
            retention_seconds,
        )?;
        ensure_new_world(&plan.world_path)?;
        let jar = store(lkjmc_store::jar::latest_matching(
            client,
            definition.jar_kind,
        ))?
        .ok_or_else(|| format!("{} jar asset not found", definition.jar_kind))?;
        let secret = read_forwarding_secret(state)?;
        let config = instance_config(state, &plan, &jar.id.to_string(), &secret)?;
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
        if let Err(error) = start_ready(state, client, &plan.instance_id, 180) {
            support::refund_purchase(client, session_id, player_uuid, cost, definition.id, &error)?;
            support::audit_event(client, &envelope, definition.id, session_id, "failed")?;
            return Err(format!("{error}; points refunded"));
        }
        store(lkjmc_store::temporary::update_session_state(
            client, session_id, "ready", None, None,
        ))?;
        support::audit_event(client, &envelope, definition.id, session_id, "succeeded")?;
        let body = support::response(
            definition,
            session_id,
            &plan.instance_id,
            ledger,
            &participants,
        );
        Ok(api::ok(envelope, body))
    })
}
