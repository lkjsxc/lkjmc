use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use lkjmc_store::audit::NewAuditEvent;
use uuid::Uuid;

use crate::app::AppState;
use crate::instance_helpers::{runtime_running, stop_runtime, store};

pub fn start_loop(state: AppState) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        if let Err(error) = run_once(&state) {
            eprintln!("temporary cleanup failed: {error}");
        }
        thread::sleep(Duration::from_secs(30));
    })
}

pub fn run_once(state: &AppState) -> Result<usize, String> {
    let Some(database_url) = state.database_url() else {
        return Ok(0);
    };
    let mut client =
        lkjmc_store::pool::connect(&database_url).map_err(|error| error.to_string())?;
    let candidates = store(lkjmc_store::temporary::cleanup_candidates(&mut client, 25))?;
    let mut count = 0;
    for candidate in candidates {
        if handle_candidate(state, &mut client, candidate)? {
            count += 1;
        }
    }
    Ok(count)
}

fn handle_candidate(
    state: &AppState,
    client: &mut postgres::Client,
    candidate: lkjmc_store::temporary::CleanupCandidate,
) -> Result<bool, String> {
    if candidate.expired && running_state(&candidate.lifecycle_state) {
        if runtime_running(state, &candidate.instance_id)? {
            stop_runtime(state, client, &candidate.instance_id)?;
        }
        store(lkjmc_store::instance::update_desired_state(
            client,
            &candidate.instance_id,
            "stopped",
        ))?;
        store(lkjmc_store::temporary::update_instance_state(
            client,
            &candidate.instance_id,
            "stopped",
            None,
        ))?;
        audit(client, &candidate.instance_id, "stopped")?;
    }
    if !candidate.cleanup_due {
        return Ok(candidate.expired);
    }
    store(lkjmc_store::temporary::update_instance_state(
        client,
        &candidate.instance_id,
        "cleaning",
        None,
    ))?;
    match cleanup_world(&candidate.world_path, &candidate.cleanup_policy) {
        Ok(final_state) => finish_cleanup(client, &candidate.instance_id, final_state),
        Err(error) => fail_cleanup(client, &candidate.instance_id, &error),
    }
}

fn finish_cleanup(
    client: &mut postgres::Client,
    id: &str,
    final_state: &'static str,
) -> Result<bool, String> {
    store(lkjmc_store::instance::release_ports(client, id))?;
    store(lkjmc_store::temporary::update_instance_state(
        client,
        id,
        final_state,
        None,
    ))?;
    store(lkjmc_store::temporary::record_cleanup_event(
        client,
        Uuid::new_v4(),
        id,
        "cleanup-worker",
        "succeeded",
        None,
    ))?;
    audit(client, id, "succeeded")?;
    Ok(true)
}

fn fail_cleanup(client: &mut postgres::Client, id: &str, error: &str) -> Result<bool, String> {
    store(lkjmc_store::temporary::update_instance_state(
        client,
        id,
        "failed",
        Some(error),
    ))?;
    store(lkjmc_store::temporary::record_cleanup_event(
        client,
        Uuid::new_v4(),
        id,
        "cleanup-worker",
        "failed",
        Some(error),
    ))?;
    audit(client, id, "failed")?;
    Ok(false)
}

fn cleanup_world(path: &str, policy: &str) -> Result<&'static str, String> {
    match policy {
        "delete" => {
            if Path::new(path).exists() {
                fs::remove_dir_all(path).map_err(|error| format!("delete world: {error}"))?;
            }
            Ok("cleaned")
        }
        "archive" => {
            if Path::new(path).exists() {
                fs::rename(path, format!("{path}.archive.{}", Uuid::new_v4()))
                    .map_err(|error| format!("archive world: {error}"))?;
            }
            Ok("archived")
        }
        other => Err(format!("unsupported cleanup policy: {other}")),
    }
}

fn running_state(state: &str) -> bool {
    matches!(state, "created" | "starting" | "ready")
}

fn audit(client: &mut postgres::Client, id: &str, result: &str) -> Result<(), String> {
    lkjmc_store::audit::insert(
        client,
        NewAuditEvent {
            id: Uuid::new_v4(),
            actor_kind: "daemon",
            actor_name: "temporary-cleanup",
            action: "temporary.instance.cleanup",
            target_kind: "temporary-instance",
            target_id: id,
            result,
        },
    )
    .map_err(|error| error.to_string())
}
