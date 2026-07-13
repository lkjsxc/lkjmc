use std::path::PathBuf;

use lkjmc_core::runtime_lifecycle::{LifecycleDecision, RuntimeIntent, RuntimeObserved};

use crate::app::AppState;
use crate::runtime::instance_launch::LaunchSpec;
use crate::runtime::reconcile::{RuntimeGoal, EFFECT_DEADLINE};
use crate::runtime::RuntimeObservation;

pub(super) struct PreparedStart {
    launch: LaunchSpec,
    work_dir: PathBuf,
}

pub(super) fn desired_intent(goal: RuntimeGoal) -> Result<RuntimeIntent, String> {
    match goal {
        RuntimeGoal::Running => Ok(RuntimeIntent::Running),
        RuntimeGoal::Stopped => Ok(RuntimeIntent::Stopped),
        RuntimeGoal::Deleted => Ok(RuntimeIntent::Deleted),
        RuntimeGoal::Observe => Err("observe has no desired intent".to_string()),
    }
}

pub(super) fn prepare_start(state: &AppState, id: &str) -> Result<PreparedStart, String> {
    let (kind, config, launch) = {
        let mut client = state.database_connection()?;
        let instance = lkjmc_store::instance::get(&mut client, id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("instance not found: {id}"))?;
        let config = lkjmc_store::instance::config(&mut client, id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("instance config not found: {id}"))?;
        let launch =
            crate::runtime::instance_launch::launch(state, &mut client, &instance.kind, &config)?;
        (instance.kind, config, launch)
    };
    let work_dir = crate::templates::render_instance(state, id, &kind, &config)?;
    Ok(PreparedStart { launch, work_dir })
}

pub(super) fn perform(
    state: &AppState,
    id: &str,
    decision: LifecycleDecision,
    prepared: Option<&PreparedStart>,
    stop_config: Option<&serde_json::Value>,
) -> Result<RuntimeObservation, String> {
    let runtime = state.runtime();
    match decision {
        LifecycleDecision::Start => {
            let prepared = prepared.ok_or("start plan missing")?;
            let observation = runtime.start(
                id,
                &prepared.launch.command,
                &prepared.launch.args,
                &prepared.launch.env,
                &state.log_root(),
                &prepared.work_dir,
                EFFECT_DEADLINE,
            )?;
            observation
                .healthy
                .then_some(observation)
                .ok_or_else(|| "process did not become healthy after start".to_string())
        }
        LifecycleDecision::Stop => {
            if let Some(config) = stop_config {
                let _ = crate::runtime::rcon::stop_from_config(config);
            }
            runtime.stop(id, EFFECT_DEADLINE)
        }
        LifecycleDecision::Delete => runtime.delete(id, EFFECT_DEADLINE),
        other => Err(format!("runtime decision cannot perform effect: {other:?}")),
    }
}

pub(super) fn observed_kind(observation: Option<&RuntimeObservation>) -> RuntimeObserved {
    match observation {
        None => RuntimeObserved::Absent,
        Some(value) if value.healthy => RuntimeObserved::Running,
        Some(value) if value.observed_state.contains("absent") => RuntimeObserved::Absent,
        Some(_) => RuntimeObserved::Unhealthy,
    }
}

pub(super) fn intent_name(intent: RuntimeIntent) -> &'static str {
    match intent {
        RuntimeIntent::Running => "running",
        RuntimeIntent::Stopped => "stopped",
        RuntimeIntent::Deleted => "deleted",
    }
}

pub(super) fn observed_name(observation: Option<&RuntimeObservation>) -> &'static str {
    match observed_kind(observation) {
        RuntimeObserved::Running => "running",
        RuntimeObserved::Absent => "absent",
        RuntimeObserved::Unhealthy => "unhealthy",
        RuntimeObserved::Unknown => "unknown",
    }
}

pub(super) fn decision_name(decision: LifecycleDecision) -> &'static str {
    match decision {
        LifecycleDecision::Start => "start",
        LifecycleDecision::Stop => "stop",
        LifecycleDecision::Delete => "delete",
        LifecycleDecision::ObservePending => "observe",
        LifecycleDecision::Noop => "observe",
        LifecycleDecision::Unsupported => "observe",
    }
}
