use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::Value;

use crate::app::AppState;
use crate::commands::adventure_api::purchase_prepare::{prepare, PreparedPurchase};
use crate::commands::adventure_api::purchase_support as support;
use crate::commands::temporary_api::lifecycle::start_ready;
use crate::dispatch as api;
use crate::support::instance_helpers::store;

pub fn end(state: &AppState, mut envelope: CommandEnvelope) -> CommandResponse {
    envelope.body["adventureId"] = Value::String("end-expedition".to_string());
    purchase(state, envelope)
}

pub fn purchase(state: &AppState, envelope: CommandEnvelope) -> CommandResponse {
    let prepared = match state
        .database_connection()
        .and_then(|mut client| prepare(state, &envelope, &mut client))
    {
        Ok(prepared) => prepared,
        Err(error) => return api::error(envelope, "adventure.error", error, false),
    };
    if let Err(error) = start_ready(state, &prepared.instance_id, 180) {
        return failed_start(state, envelope, prepared, error);
    }
    finish_purchase(state, envelope, prepared)
}

fn failed_start(
    state: &AppState,
    envelope: CommandEnvelope,
    prepared: PreparedPurchase,
    error: String,
) -> CommandResponse {
    let result = state.database_connection().and_then(|mut client| {
        support::refund_purchase(
            &mut client,
            prepared.session_id,
            &prepared.adventure_id,
            &error,
        )?;
        support::audit_event(
            &mut client,
            &envelope,
            &prepared.adventure_id,
            prepared.session_id,
            "failed",
        )
    });
    match result {
        Ok(()) => api::error(
            envelope,
            "adventure.error",
            format!("{error}; points refunded"),
            false,
        ),
        Err(refund_error) => api::error(envelope, "adventure.refund_failed", refund_error, false),
    }
}

fn finish_purchase(
    state: &AppState,
    envelope: CommandEnvelope,
    prepared: PreparedPurchase,
) -> CommandResponse {
    let result = state.database_connection().and_then(|mut client| {
        store(lkjmc_store::temporary::update_session_state(
            &mut *client,
            prepared.session_id,
            "ready",
            None,
            None,
        ))?;
        support::audit_event(
            &mut client,
            &envelope,
            &prepared.adventure_id,
            prepared.session_id,
            "succeeded",
        )
    });
    let definition = lkjmc_core::adventure::get(&prepared.adventure_id);
    match (result, definition) {
        (Ok(()), Some(definition)) => api::ok(
            envelope,
            support::response(
                definition,
                prepared.session_id,
                &prepared.instance_id,
                prepared.ledger,
                &prepared.participants,
            ),
        ),
        (Err(error), _) => api::error(envelope, "adventure.error", error, false),
        (_, None) => api::error(
            envelope,
            "adventure.error",
            "adventure was withdrawn",
            false,
        ),
    }
}
