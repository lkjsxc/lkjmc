use std::thread;
use std::time::Duration;

use crate::app::AppState;
use crate::instance_helpers::{
    refresh_runtime, runtime_running, start_runtime, stop_runtime, store, write_observation,
};
use crate::runtime::RuntimeObservation;

pub fn recover(state: &AppState) -> Result<(), String> {
    let Some(database_url) = &state.database_url else {
        return Ok(());
    };
    let mut client = lkjmc_store::pool::connect(database_url).map_err(|error| error.to_string())?;
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
    let Some(database_url) = &state.database_url else {
        return Ok(());
    };
    let mut client = lkjmc_store::pool::connect(database_url).map_err(|error| error.to_string())?;
    refresh_runtime(state, &mut client)?;
    for row in store(lkjmc_store::instance::list(&mut client))? {
        reconcile_instance(state, &mut client, &row.id, &row.desired_state)?;
    }
    Ok(())
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
        "deleting" | "failed" => Ok(()),
        _ => Ok(()),
    }
}

fn ensure_running(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
    if runtime_running(state, id)? {
        return Ok(());
    }
    match start_runtime(state, client, id) {
        Ok(_) => Ok(()),
        Err(error) => {
            let observation = RuntimeObservation::unhealthy(error);
            write_observation(client, id, &observation)
        }
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
