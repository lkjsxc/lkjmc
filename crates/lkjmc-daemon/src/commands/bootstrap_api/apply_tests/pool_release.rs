#[path = "pool_release/fixture.rs"]
mod fixture;

use std::sync::mpsc;
use std::time::Duration;

use lkjmc_core::id::InstanceId;
use serde_json::json;
use uuid::Uuid;

use super::super::{effects, network_plan::NetworkEffect, readiness_wait};
use fixture::{BlockAt, Harness};

#[test]
fn size_one_pool_is_available_while_render_blocks() -> Result<(), String> {
    let Some((harness, _, _)) = Harness::new(BlockAt::Never)? else {
        return Ok(());
    };
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let state = harness.state.clone();
    let worker = std::thread::spawn(move || {
        effects::render_with(&state, "pool-probe", |_, _| {
            entered_tx.send(()).map_err(|error| error.to_string())?;
            release_rx.recv().map_err(|error| error.to_string())
        })
    });
    assert_released(&harness, entered_rx, release_tx)?;
    worker.join().map_err(|_| "render worker panicked")??;
    Ok(())
}

#[test]
fn size_one_pool_is_available_while_runtime_start_blocks() -> Result<(), String> {
    let Some((harness, entered, release)) = Harness::new(BlockAt::Start)? else {
        return Ok(());
    };
    let state = harness.state.clone();
    let worker = std::thread::spawn(move || {
        effects::apply_runtime_effect(
            &state,
            &NetworkEffect::StartInstance {
                id: InstanceId::internal("pool-probe"),
            },
        )
    });
    assert_released(&harness, entered, release)?;
    assert!(worker.join().map_err(|_| "start worker panicked")?.is_err());
    Ok(())
}

#[test]
fn size_one_pool_is_available_while_runtime_observation_blocks() -> Result<(), String> {
    let Some((mut harness, entered, release)) = Harness::new(BlockAt::Status)? else {
        return Ok(());
    };
    let run_id = Uuid::new_v4();
    lkjmc_store::bootstrap::create_run(
        harness.database.client_mut(),
        lkjmc_store::bootstrap::NewBootstrapRun {
            id: run_id,
            profile: "playable",
            requested_by: "pool-test",
            result: "running",
            diagnostics: json!([]),
        },
    )
    .map_err(|error| error.to_string())?;
    let state = harness.state.clone();
    let worker = std::thread::spawn(move || {
        readiness_wait::run(
            &state,
            Uuid::nil(),
            run_id,
            0,
            &NetworkEffect::WaitForReadiness {
                id: InstanceId::internal("pool-probe"),
            },
            "pool-probe",
        )
    });
    assert_released(&harness, entered, release)?;
    assert!(worker
        .join()
        .map_err(|_| "readiness worker panicked")?
        .is_err());
    Ok(())
}

fn assert_released(
    harness: &Harness,
    entered: mpsc::Receiver<()>,
    release: mpsc::Sender<()>,
) -> Result<(), String> {
    entered
        .recv_timeout(Duration::from_secs(2))
        .map_err(|error| error.to_string())?;
    let available = harness.state.database_connection();
    release.send(()).map_err(|error| error.to_string())?;
    assert!(
        available.is_ok(),
        "size-one pool was held during an external effect"
    );
    Ok(())
}
