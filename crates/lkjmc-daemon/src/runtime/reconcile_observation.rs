use lkjmc_core::runtime_lifecycle::RuntimeIntent;
use lkjmc_store::runtime_adoption::{self, PendingRuntimeOperation, RuntimeOperation};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::runtime::reconcile_plan::intent_name;
use crate::runtime::RuntimeObservation;

pub(super) fn observe_locked(
    state: &AppState,
    id: &str,
    correlation_id: Uuid,
    persisted: Option<&serde_json::Value>,
) -> Result<RuntimeObservation, String> {
    let operation = {
        let mut client = state.database_connection()?;
        runtime_adoption::allocate(
            &mut client,
            id,
            "observe",
            &json!({"desired":"observe"}),
            correlation_id,
        )
        .map_err(|error| error.to_string())?
    };
    let owned = {
        let mut client = state.database_connection()?;
        runtime_adoption::mark_effect(&mut client, &operation).map_err(|error| error.to_string())?
    };
    if !owned {
        return Err("runtime observe fence ownership changed".to_string());
    }
    let observation = observe_adapter(state, id, persisted)?
        .unwrap_or_else(|| RuntimeObservation::absent("runtime is absent"));
    finish(state, &operation, &observation, "succeeded", None)?;
    Ok(observation)
}

pub(super) fn record_noop(
    state: &AppState,
    id: &str,
    correlation_id: Uuid,
    intent: RuntimeIntent,
    observation: &RuntimeObservation,
) -> Result<(), String> {
    let operation = {
        let mut client = state.database_connection()?;
        runtime_adoption::allocate(
            &mut client,
            id,
            "observe",
            &json!({"desired":intent_name(intent),"decision":"noop"}),
            correlation_id,
        )
        .map_err(|error| error.to_string())?
    };
    finish(state, &operation, observation, "noop", None)
}

pub(super) struct PendingRepair {
    pub observation: RuntimeObservation,
    pub outcome: &'static str,
}

pub(super) fn repair_pending(
    state: &AppState,
    id: &str,
    pending: &PendingRuntimeOperation,
) -> Result<Option<PendingRepair>, String> {
    let observation = observe_adapter(state, id, pending.observation.as_ref())?;
    Ok(match (pending.operation.intent.as_str(), observation) {
        ("start", Some(value)) if value.healthy => Some(PendingRepair {
            observation: value,
            outcome: "succeeded",
        }),
        ("start", None) => Some(PendingRepair {
            observation: RuntimeObservation::absent("start effect is proved absent"),
            outcome: "failed",
        }),
        ("stop" | "delete", None) => Some(PendingRepair {
            observation: RuntimeObservation::absent("effect absence observed"),
            outcome: "succeeded",
        }),
        ("stop" | "delete", Some(value)) if value.observed_state.contains("absent") => {
            Some(PendingRepair {
                observation: value,
                outcome: "succeeded",
            })
        }
        _ => None,
    })
}

pub(super) fn observe_adapter(
    state: &AppState,
    id: &str,
    persisted: Option<&serde_json::Value>,
) -> Result<Option<RuntimeObservation>, String> {
    let runtime = state.runtime();
    let current = runtime.runtime_status(id)?;
    if current.is_some() {
        return Ok(current);
    }
    let Some(identity) = persisted.and_then(RuntimeObservation::identity_from_json) else {
        return Ok(None);
    };
    runtime.runtime_adopt(id, identity)?;
    runtime.runtime_status(id)
}

pub(super) fn finish(
    state: &AppState,
    operation: &RuntimeOperation,
    observation: &RuntimeObservation,
    outcome: &str,
    detail: Option<&str>,
) -> Result<(), String> {
    let mut client = state.database_connection()?;
    runtime_adoption::observe(
        &mut client,
        operation,
        &observation.to_json(),
        outcome,
        detail,
    )
    .map_err(|error| error.to_string())?
    .then_some(())
    .ok_or_else(|| "runtime fence ownership changed after effect".to_string())
}
