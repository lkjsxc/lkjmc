use lkjmc_core::config::NetworkConfig;
use lkjmc_core::instance::DesiredState;
use lkjmc_store::network_intent::ApplyAttempt;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::runtime::RuntimeGoal;

pub(super) fn recover(state: &AppState) -> Result<(), String> {
    let current = state
        .runtime_config()?
        .ok_or("runtime config is unavailable")?
        .network;
    let candidates = {
        let mut client = state.database_connection()?;
        super::network_record::ensure_migrations(&mut client)?;
        lkjmc_store::network_intent::recovery_candidates(&mut client)
            .map_err(|error| error.to_string())?
    };
    for attempt in candidates {
        if !matches!(attempt.effect_phase.as_str(), "runtime" | "observation") {
            fail_known_pre_runtime(state, &attempt)?;
            continue;
        }
        ensure_unknown(state, &attempt)?;
        let observation = match reconcile_owned(state, &attempt, &current) {
            Ok(value) => value,
            Err(error) => {
                return Err(format!(
                    "network attempt {} remains unknown: {error}",
                    attempt.id
                ));
            }
        };
        let mut client = state.database_connection()?;
        lkjmc_store::network_intent::complete_unknown(&mut client, attempt.id, &observation)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn fail_known_pre_runtime(state: &AppState, attempt: &ApplyAttempt) -> Result<(), String> {
    if attempt.outcome == "unknown" {
        return Err("unknown network attempt has no runtime phase".to_string());
    }
    let observation = json!({
        "recoveryComplete": true,
        "runtimeEffectPossible": false,
        "rollbackClaimed": false,
    });
    let mut client = state.database_connection()?;
    lkjmc_store::network_intent::finish_attempt(
        &mut client,
        attempt.id,
        "failed",
        Some("interrupted before any runtime effect"),
        &observation,
    )
    .map_err(|error| error.to_string())
}

fn ensure_unknown(state: &AppState, attempt: &ApplyAttempt) -> Result<(), String> {
    if attempt.outcome == "unknown" {
        return Ok(());
    }
    let observation = json!({
        "recoveryComplete": false,
        "runtimeEffectPossible": true,
        "rollbackClaimed": false,
    });
    let mut client = state.database_connection()?;
    lkjmc_store::network_intent::finish_attempt(
        &mut client,
        attempt.id,
        "unknown",
        Some("runtime effect may have occurred; adapter observation required"),
        &observation,
    )
    .map_err(|error| error.to_string())
}

fn reconcile_owned(
    state: &AppState,
    attempt: &ApplyAttempt,
    current: &NetworkConfig,
) -> Result<Value, String> {
    let old = desired_network(state, attempt.network_revision)?;
    let mut resources = Map::new();
    for instance in old.instances {
        let observed = crate::runtime::reconcile::reconcile(
            state,
            &instance.id,
            RuntimeGoal::Observe,
            Uuid::new_v4(),
        )?;
        let running = current
            .instances
            .iter()
            .find(|value| value.id == instance.id)
            .is_some_and(|value| value.desired_state == DesiredState::Running);
        let reconciled = crate::runtime::reconcile::reconcile(
            state,
            &instance.id,
            if running {
                RuntimeGoal::Running
            } else {
                RuntimeGoal::Stopped
            },
            Uuid::new_v4(),
        )?;
        validate_goal(running, &reconciled)?;
        resources.insert(
            instance.id,
            json!({"observed": observed.to_json(), "reconciled": reconciled.to_json()}),
        );
    }
    Ok(json!({
        "recoveryComplete": true,
        "runtimeEffectPossible": true,
        "rollbackClaimed": false,
        "resources": resources,
    }))
}

fn desired_network(state: &AppState, revision: i64) -> Result<NetworkConfig, String> {
    let desired = {
        let mut client = state.database_connection()?;
        lkjmc_store::network_intent::desired_by_revision(&mut client, revision)
            .map_err(|error| error.to_string())?
            .ok_or("network recovery intent is absent")?
    };
    serde_json::from_value(desired.intent).map_err(|error| error.to_string())
}

fn validate_goal(
    running: bool,
    observation: &crate::runtime::RuntimeObservation,
) -> Result<(), String> {
    if (running && observation.healthy)
        || (!running && observation.observed_state.contains("absent"))
    {
        Ok(())
    } else if running {
        Err("owned runtime was not observed running".to_string())
    } else {
        Err("owned runtime was not observed absent".to_string())
    }
}
