use std::thread;
use std::time::Duration;

use lkjmc_core::autosuspend::{self, AutosuspendDecision, AutosuspendInput};
use serde_json::Value;

use crate::reconcile::policy::{desired, kind, policy};

use crate::app::AppState;
use crate::runtime::RuntimeObservation;
use crate::support::instance_helpers::{
    refresh_runtime, runtime_running, start_runtime, stop_runtime, store, write_observation,
};

pub fn recover(state: &AppState) -> Result<(), String> {
    if state.database_url().is_none() {
        return Ok(());
    }
    let mut client = state.database_connection()?;
    let instances = store(lkjmc_store::instance::list(&mut client))?;
    let mut runtime = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned".to_string())?;
    for row in instances {
        let Some(pid) = row.pid.and_then(|pid| u32::try_from(pid).ok()) else {
            continue;
        };
        if row.healthy == Some(true) {
            let observation = runtime.recover(&row.id, pid);
            write_observation(&mut client, &row.id, &observation)?;
        }
    }
    Ok(())
}

pub fn start_loop(state: AppState) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        if let Err(error) = tick(&state) {
            eprintln!("reconciler tick failed: {error}");
        }
        thread::sleep(Duration::from_secs(1));
    })
}

fn tick(state: &AppState) -> Result<(), String> {
    if state.database_url().is_none() {
        return Ok(());
    }
    let mut client = state.database_connection()?;
    refresh_runtime(state, &mut client)?;
    for row in store(lkjmc_store::instance::list(&mut client))? {
        if maybe_autosuspend(state, &mut client, &row)? {
            continue;
        }
        reconcile_instance(state, &mut client, &row.id, &row.desired_state)?;
    }
    Ok(())
}

fn maybe_autosuspend(
    state: &AppState,
    client: &mut postgres::Client,
    row: &lkjmc_store::instance::InstanceRecord,
) -> Result<bool, String> {
    let Some(kind) = kind(&row.kind) else {
        return Ok(false);
    };
    let Some(desired_state) = desired(&row.desired_state) else {
        return Ok(false);
    };
    let Some(presence) = store(lkjmc_store::instance_presence::get(client, &row.id))? else {
        return Ok(false);
    };
    let active = store(lkjmc_store::player_session::active_count_for_server(
        client, &row.id,
    ))?;
    let config = store(lkjmc_store::instance::config(client, &row.id))?.unwrap_or(Value::Null);
    let policy = policy(kind, &row.id, config.get("autosuspend"));
    let empty_count =
        if presence.player_count == Some(0) && presence.empty_since_age_seconds.is_some() {
            policy.empty_heartbeat_count
        } else if presence.player_count == Some(0) {
            1
        } else {
            0
        };
    let input = AutosuspendInput {
        kind,
        desired_state,
        observed_running: row.healthy.unwrap_or(false) && runtime_running(state, &row.id)?,
        heartbeat_age_seconds: presence
            .heartbeat_age_seconds
            .and_then(|v| u64::try_from(v).ok()),
        player_count: presence.player_count.and_then(|v| u32::try_from(v).ok()),
        active_sessions: u32::try_from(active).unwrap_or(u32::MAX),
        uptime_seconds: row.uptime_seconds.and_then(|v| u64::try_from(v).ok()),
        empty_since_age_seconds: presence
            .empty_since_age_seconds
            .and_then(|v| u64::try_from(v).ok()),
        consecutive_empty_heartbeats: empty_count,
        policy,
    };
    match autosuspend::plan(input) {
        AutosuspendDecision::SetEmptySince => store(
            lkjmc_store::instance_presence::set_empty_since(client, &row.id),
        )
        .map(|_| false),
        AutosuspendDecision::ClearEmptySince => store(
            lkjmc_store::instance_presence::clear_empty_since(client, &row.id),
        )
        .map(|_| false),
        AutosuspendDecision::MarkSuspendedAndStop { reason } => {
            store(lkjmc_store::instance_presence::mark_autosuspended(
                client, &row.id, &reason,
            ))?;
            stop_runtime(state, client, &row.id)?;
            Ok(true)
        }
        AutosuspendDecision::Noop | AutosuspendDecision::Skip { .. } => Ok(false),
    }
}

fn reconcile_instance(
    state: &AppState,
    client: &mut postgres::Client,
    id: &str,
    desired_state: &str,
) -> Result<(), String> {
    match desired_state {
        "running" | "starting" => ensure_running(state, client, id),
        "stopped" | "stopping" => ensure_stopped(state, client, id),
        "restarting" => restart(state, client, id),
        "suspended" | "deleting" | "failed" => Ok(()),
        _ => Ok(()),
    }
}

fn ensure_running(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
    if runtime_running(state, id)? {
        return Ok(());
    }
    match start_runtime(state, client, id) {
        Ok(_) => Ok(()),
        Err(error) => write_observation(client, id, &RuntimeObservation::unhealthy(error)),
    }
}

fn ensure_stopped(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
    if runtime_running(state, id)? {
        stop_runtime(state, client, id)?;
    }
    Ok(())
}

fn restart(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
    stop_runtime(state, client, id)?;
    start_runtime(state, client, id)?;
    store(lkjmc_store::instance::update_desired_state(
        client, id, "running",
    ))?;
    Ok(())
}

#[cfg(test)]
#[path = "reconciler_tests.rs"]
mod tests;
