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
    let config = state
        .runtime_config()?
        .ok_or("runtime config is unavailable")?;
    let intent = serde_json::to_value(&config.network).map_err(|error| error.to_string())?;
    let mut client = state.database_connection()?;
    ensure_migrations(&mut client)?;
    let (_desired, attempt) = lkjmc_store::network_intent::record_desired_with_attempt(
        &mut client,
        i64::try_from(config.network.revision)
            .map_err(|_| "network revision exceeds PostgreSQL bigint")?,
        &inspection.intent_digest,
        &intent,
        request.request_id.as_str(),
    )
    .map_err(|error| error.to_string())?;
    match inspection.outcome {
        InspectionOutcome::Blocked => {
            let diagnostic = inspection.unsupported.join("; ");
            lkjmc_store::network_intent::finish_attempt(
                &mut client,
                attempt.id,
                "unsupported",
                Some(&diagnostic),
                &json!({}),
            )
            .map_err(|error| error.to_string())?;
            Ok(Admission::Unsupported(attempt.id, diagnostic))
        }
        InspectionOutcome::NoOp => {
            lkjmc_store::network_intent::finish_attempt(
                &mut client,
                attempt.id,
                "no-op",
                None,
                &json!({"intentDigest": inspection.intent_digest}),
            )
            .map_err(|error| error.to_string())?;
            Ok(Admission::NoOp(attempt.id))
        }
        InspectionOutcome::Changes => {
            lkjmc_store::network_intent::mark_applying(&mut client, attempt.id)
                .map_err(|error| error.to_string())?;
            Ok(Admission::Applying(attempt.id))
        }
    }
}

pub(super) fn ensure_migrations(client: &mut postgres::Client) -> Result<(), String> {
    let table_ready: bool = client
        .query_one(
            "select to_regclass(current_schema() || '.schema_migrations') is not null",
            &[],
        )
        .map_err(|error| error.to_string())?
        .get(0);
    let current = table_ready
        && client
            .query_opt("select 1 from schema_migrations where version = 49", &[])
            .map_err(|error| error.to_string())?
            .is_some();
    if current {
        Ok(())
    } else {
        lkjmc_store::migrate::apply(client)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

pub(super) fn mark_phase(state: &AppState, id: Uuid, phase: &str) -> Result<(), String> {
    let mut client = state.database_connection()?;
    mark_phase_with_client(&mut client, id, phase)
}

pub(super) fn mark_phase_with_client(
    client: &mut postgres::Client,
    id: Uuid,
    phase: &str,
) -> Result<(), String> {
    lkjmc_store::network_intent::mark_effect_phase(client, id, phase)
        .map_err(|error| error.to_string())
}

pub(super) fn finish_error(state: &AppState, id: Uuid, diagnostic: &str) -> Result<(), String> {
    let mut client = state.database_connection()?;
    let attempt = lkjmc_store::network_intent::attempt(&mut client, id)
        .map_err(|error| error.to_string())?
        .ok_or("network attempt is absent")?;
    let possible = matches!(attempt.effect_phase.as_str(), "runtime" | "observation");
    let outcome = if possible { "unknown" } else { "failed" };
    let observation = json!({
        "recoveryComplete": !possible,
        "runtimeEffectPossible": possible,
        "rollbackClaimed": false,
    });
    lkjmc_store::network_intent::finish_attempt(
        &mut client,
        id,
        outcome,
        Some(diagnostic),
        &observation,
    )
    .map_err(|error| error.to_string())
}

pub(super) fn finish(
    state: &AppState,
    id: Uuid,
    outcome: &str,
    diagnostic: Option<&str>,
    observation: Value,
) -> Result<(), String> {
    let mut client = state.database_connection()?;
    lkjmc_store::network_intent::finish_attempt(&mut client, id, outcome, diagnostic, &observation)
        .map_err(|error| error.to_string())
}
