use lkjmc_core::command::CommandEnvelope;
use lkjmc_core::network_intent::{InspectionOutcome, NetworkInspection};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;

pub(super) enum Admission {
    Applying(Uuid),
    NoOp(Uuid),
    Unsupported(Uuid, String),
}

pub(super) fn admit(
    state: &AppState,
    request: &CommandEnvelope,
    inspection: &NetworkInspection,
) -> Result<Admission, String> {
    let config = state.runtime_config()?.ok_or("runtime config is unavailable")?;
    let intent = serde_json::to_value(&config.network).map_err(|error| error.to_string())?;
    let mut client = state.database_connection()?;
    lkjmc_store::migrate::apply(&mut client).map_err(|error| error.to_string())?;
    let desired = lkjmc_store::network_intent::record_desired(
        &mut client,
        i64::try_from(config.network.revision).map_err(|_| "network revision exceeds PostgreSQL bigint")?,
        &inspection.intent_digest,
        &intent,
        request.request_id.as_str(),
    ).map_err(|error| error.to_string())?;
    let attempt = lkjmc_store::network_intent::create_attempt(
        &mut client, desired.revision, request.request_id.as_str(),
    ).map_err(|error| error.to_string())?;
    match inspection.outcome {
        InspectionOutcome::Blocked => {
            let diagnostic = inspection.unsupported.join("; ");
            lkjmc_store::network_intent::finish_attempt(
                &mut client, attempt.id, "unsupported", Some(&diagnostic), &json!({}),
            ).map_err(|error| error.to_string())?;
            Ok(Admission::Unsupported(attempt.id, diagnostic))
        }
        InspectionOutcome::NoOp => {
            lkjmc_store::network_intent::finish_attempt(
                &mut client, attempt.id, "no-op", None, &json!({"intentDigest": inspection.intent_digest}),
            ).map_err(|error| error.to_string())?;
            Ok(Admission::NoOp(attempt.id))
        }
        InspectionOutcome::Changes => {
            lkjmc_store::network_intent::mark_applying(&mut client, attempt.id)
                .map_err(|error| error.to_string())?;
            Ok(Admission::Applying(attempt.id))
        }
    }
}

pub(super) fn finish(
    state: &AppState,
    id: Uuid,
    outcome: &str,
    diagnostic: Option<&str>,
    observation: Value,
) -> Result<(), String> {
    let mut client = state.database_connection()?;
    lkjmc_store::network_intent::finish_attempt(
        &mut client, id, outcome, diagnostic, &observation,
    ).map_err(|error| error.to_string())
}
