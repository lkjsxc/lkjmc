use std::sync::{mpsc, Arc, Barrier};
use std::time::Duration;

use super::adoption_tests::Fixture;
use super::process;
use super::test_support::{unique_id, StateCleanup};
use crate::support::instance_helpers::{start_runtime, stop_runtime};

#[test]
fn cross_instance_database_process_hang() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new()? else {
        return Ok(());
    };
    let held_id = unique_id("held");
    let peer_id = unique_id("peer");
    fixture.insert(&peer_id)?;
    let state = Arc::new(fixture.state());
    let _cleanup = StateCleanup(Arc::clone(&state));
    let held_state = Arc::clone(&state);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let holder = std::thread::spawn(move || {
        held_state.coordinate_runtime(&held_id, || {
            use std::os::unix::process::CommandExt;
            let mut command = std::process::Command::new("sleep");
            command.arg("30").process_group(0);
            let mut child = command.spawn().map_err(|error| error.to_string())?;
            entered_tx.send(()).map_err(|error| error.to_string())?;
            let result = release_rx.recv().map_err(|error| error.to_string());
            let _ = child.kill();
            let _ = child.wait();
            result
        })
    });
    entered_rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| "hung instance did not enter effect".to_string())?;
    let peer_state = Arc::clone(&state);
    let peer_effect_id = peer_id.clone();
    let (peer_tx, peer_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = peer_tx.send(start_runtime(&peer_state, &peer_effect_id));
    });
    let observation = peer_rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| "peer database/process effect was blocked".to_string())??;
    let pid = observation.pid().ok_or("peer process identity missing")?;
    release_tx.send(()).map_err(|error| error.to_string())?;
    holder.join().map_err(|_| "holder panicked".to_string())??;
    stop_runtime(&state, &peer_id)?;
    state.shutdown_runtime()?;
    assert!(!process::group_exists(pid));
    Ok(())
}

#[test]
fn same_instance_database_process_race() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new()? else {
        return Ok(());
    };
    let race_id = unique_id("race");
    fixture.insert(&race_id)?;
    let state = Arc::new(fixture.state());
    let _cleanup = StateCleanup(Arc::clone(&state));
    let barrier = Arc::new(Barrier::new(3));
    let (done_tx, done_rx) = mpsc::channel();
    let start_state = Arc::clone(&state);
    let start_id = race_id.clone();
    let start_barrier = Arc::clone(&barrier);
    let start_done = done_tx.clone();
    let start = std::thread::spawn(move || {
        start_barrier.wait();
        let _ = start_done.send(start_runtime(&start_state, &start_id).map(|_| ()));
    });
    let stop_state = Arc::clone(&state);
    let stop_id = race_id.clone();
    let stop_barrier = Arc::clone(&barrier);
    let stop = std::thread::spawn(move || {
        stop_barrier.wait();
        let _ = done_tx.send(stop_runtime(&stop_state, &stop_id).map(|_| ()));
    });
    barrier.wait();
    for _ in 0..2 {
        done_rx
            .recv_timeout(Duration::from_secs(4))
            .map_err(|_| "same-instance database/process race timed out".to_string())??;
    }
    start
        .join()
        .map_err(|_| "start racer panicked".to_string())?;
    stop.join().map_err(|_| "stop racer panicked".to_string())?;
    let starts: i64 = fixture
        .database
        .client_mut()
        .query_one(
            "select count(*) from runtime_effect_workflows where instance_id=$1 and effect_kind='start'",
            &[&race_id],
        )
        .map_err(|error| error.to_string())?
        .get(0);
    assert!(starts <= 1);
    let observation = start_runtime(&state, &race_id)?;
    let pid = observation.pid().ok_or("race process identity missing")?;
    state.shutdown_runtime()?;
    assert!(!process::group_exists(pid));
    Ok(())
}
