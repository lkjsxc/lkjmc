use std::time::Duration;

use lkjmc_core::runtime_lifecycle::{decide, LifecycleDecision, LifecycleInput};
use lkjmc_store::runtime_adoption::{self, PendingRuntimeOperation, RuntimeOperation};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::runtime::reconcile_observation::{
    finish, observe_adapter, observe_locked, record_noop, repair_pending,
};
use crate::runtime::reconcile_plan::{
    decision_name, desired_intent, intent_name, observed_kind, observed_name, perform,
    prepare_start, PreparedStart,
};
use crate::runtime::RuntimeObservation;

pub(super) const EFFECT_DEADLINE: Duration = Duration::from_secs(8);

#[derive(Clone, Copy)]
pub enum RuntimeGoal {
    Running,
    Stopped,
    Deleted,
    Observe,
}

pub fn reconcile(
    state: &AppState,
    id: &str,
    goal: RuntimeGoal,
    correlation_id: Uuid,
) -> Result<RuntimeObservation, String> {
    state.coordinate_runtime(id, || reconcile_locked(state, id, goal, correlation_id))
}

fn reconcile_locked(
    state: &AppState,
    id: &str,
    goal: RuntimeGoal,
    correlation_id: Uuid,
) -> Result<RuntimeObservation, String> {
    state.runtime().check_capabilities()?;
    let pending = {
        let mut client = state.database_connection()?;
        runtime_adoption::pending(&mut client, id).map_err(|error| error.to_string())?
    };
    if let Some(pending) = pending {
        if !pending.effect_started {
            return resume_unstarted(state, id, &pending);
        }
        if let Some(observation) = repair_pending(state, id, &pending)? {
            finish(state, &pending.operation, &observation, "succeeded", None)?;
        } else {
            return Ok(RuntimeObservation::unhealthy(
                "pending runtime effect has unknown outcome; observation required",
            ));
        }
    }

    let persisted = {
        let mut client = state.database_connection()?;
        runtime_adoption::latest_observation(&mut client, id).map_err(|error| error.to_string())?
    };
    if matches!(goal, RuntimeGoal::Observe) {
        return observe_locked(state, id, correlation_id, persisted.as_ref());
    }
    let intent = desired_intent(goal)?;
    let observed = observe_adapter(state, id, persisted.as_ref())?;
    let decision = decide(LifecycleInput {
        intent,
        observed: observed_kind(observed.as_ref()),
        pending_operation: false,
        capability_supported: true,
    });
    if decision == LifecycleDecision::Noop {
        let observation =
            observed.unwrap_or_else(|| RuntimeObservation::absent("runtime is absent"));
        record_noop(state, id, correlation_id, intent, &observation)?;
        return Ok(observation);
    }
    let prepared = if decision == LifecycleDecision::Start {
        Some(prepare_start(state, id)?)
    } else {
        None
    };
    let stop_config = if decision == LifecycleDecision::Stop {
        let mut client = state.database_connection()?;
        lkjmc_store::instance::config(&mut client, id).map_err(|error| error.to_string())?
    } else {
        None
    };
    let requested = json!({
        "desired": intent_name(intent),
        "observed": observed_name(observed.as_ref()),
        "decision": decision_name(decision),
    });
    let operation = {
        let mut client = state.database_connection()?;
        let operation = runtime_adoption::allocate(
            &mut client,
            id,
            decision_name(decision),
            &requested,
            correlation_id,
        )
        .map_err(|error| error.to_string())?;
        if operation.replay
            && !runtime_adoption::is_pending(&mut client, &operation)
                .map_err(|error| error.to_string())?
        {
            return Ok(observed.unwrap_or_else(|| RuntimeObservation::absent("runtime is absent")));
        }
        operation
    };
    execute(
        state,
        id,
        &operation,
        decision,
        prepared.as_ref(),
        stop_config.as_ref(),
    )
}

fn resume_unstarted(
    state: &AppState,
    id: &str,
    pending: &PendingRuntimeOperation,
) -> Result<RuntimeObservation, String> {
    let decision = match pending.operation.intent.as_str() {
        "start" => LifecycleDecision::Start,
        "stop" => LifecycleDecision::Stop,
        "delete" => LifecycleDecision::Delete,
        "observe" => return observe_locked(state, id, pending.operation.correlation_id, None),
        other => return Err(format!("invalid pending runtime intent: {other}")),
    };
    let prepared = (decision == LifecycleDecision::Start)
        .then(|| prepare_start(state, id))
        .transpose()?;
    let stop_config = if decision == LifecycleDecision::Stop {
        let mut client = state.database_connection()?;
        lkjmc_store::instance::config(&mut client, id).map_err(|error| error.to_string())?
    } else {
        None
    };
    execute(
        state,
        id,
        &pending.operation,
        decision,
        prepared.as_ref(),
        stop_config.as_ref(),
    )
}

fn execute(
    state: &AppState,
    id: &str,
    operation: &RuntimeOperation,
    decision: LifecycleDecision,
    prepared: Option<&PreparedStart>,
    stop_config: Option<&serde_json::Value>,
) -> Result<RuntimeObservation, String> {
    let owned = {
        let mut client = state.database_connection()?;
        runtime_adoption::mark_effect(&mut client, operation).map_err(|error| error.to_string())?
    };
    if !owned {
        return Err("runtime fence ownership changed before effect".to_string());
    }
    match perform(state, id, decision, prepared, stop_config) {
        Ok(observation) => {
            finish(state, operation, &observation, "succeeded", None)?;
            Ok(observation)
        }
        Err(error) => {
            let observation = observe_adapter(state, id, None)?.unwrap_or_else(|| {
                RuntimeObservation::unhealthy("runtime effect outcome is unknown")
            });
            finish(state, operation, &observation, "unknown", Some(&error))?;
            Err(error)
        }
    }
}
