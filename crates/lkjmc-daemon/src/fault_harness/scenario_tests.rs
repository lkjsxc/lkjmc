use std::collections::BTreeSet;

use super::adapters::{ProcessAdapter, ServiceAdapter, TransactionAdapter};
use super::control::{Boundary, Failpoints, MAX_TRANSCRIPT_ITEMS};
use super::scenarios::{FailureTranscript, ScenarioRunner, ScenarioState};

#[test]
fn transaction_boundaries_are_controllable() {
    let mut faults = Failpoints::default();
    let mut committed = BTreeSet::new();
    faults.arm(Boundary::BeforeTransactionCommit);
    assert_eq!(
        TransactionAdapter::new(&mut faults, &mut committed).commit("request-a"),
        Err(Boundary::BeforeTransactionCommit)
    );
    assert!(committed.is_empty());

    let mut faults = Failpoints::default();
    faults.arm(Boundary::AfterTransactionCommit);
    assert_eq!(
        TransactionAdapter::new(&mut faults, &mut committed).commit("request-b"),
        Err(Boundary::AfterTransactionCommit)
    );
    assert!(committed.contains("request-b"));
    assert_eq!(faults.hits().len(), 2);
}

#[test]
fn effect_boundaries_are_controllable() {
    for boundary in [
        Boundary::BeforeProcessEffect,
        Boundary::AfterProcessEffect,
        Boundary::BeforeObservation,
    ] {
        let mut faults = Failpoints::default();
        faults.arm(boundary);
        let mut process = ProcessAdapter::new(&mut faults);
        assert_eq!(process.start("alpha"), Err(boundary));
        assert_eq!(
            process.has_process("alpha"),
            boundary != Boundary::BeforeProcessEffect
        );
        assert!(!process.has_observation("alpha"));
    }
}

#[test]
fn deadline_scenario_controls_http_credential_and_shutdown() {
    for boundary in [
        Boundary::HttpDeadline,
        Boundary::CredentialLookup,
        Boundary::BeforeShutdown,
    ] {
        let mut faults = Failpoints::default();
        faults.arm(boundary);
        let mut service = ServiceAdapter::new(&mut faults);
        let result = if boundary == Boundary::HttpDeadline {
            service.await_http()
        } else if boundary == Boundary::CredentialLookup {
            service.credential()
        } else {
            service.shutdown()
        };
        assert_eq!(result, Err(boundary));
    }

    let mut runner = ScenarioRunner::new(17, 100);
    assert!(runner.database_delay_exceeds(110, 11));
}

#[test]
fn cross_instance_scenario_survives_hang_and_restart() {
    let mut runner = ScenarioRunner::new(23, 0);
    assert!(!runner.duplicate_request("request-a"));
    assert!(runner.duplicate_request("request-a"));
    runner.plugin_disconnect();
    assert!(!runner.acknowledge_plugin());
    let restarted = runner.restart();
    assert!(restarted.has_durable_request("request-a"));
    assert!(!restarted.acknowledge_plugin());
    assert_eq!(
        restarted.run_instances("alpha", &["alpha", "beta"]),
        vec!["beta"]
    );
}

fn seeded_failure(seed: u64) -> Result<ScenarioState, FailureTranscript> {
    let mut runner = ScenarioRunner::new(seed, 500);
    runner.arm(Boundary::AfterProcessEffect);
    runner.run_seeded_instances(&["alpha", "beta", "gamma"])
}

#[test]
fn deterministic_seed_replay_reproduces_armed_failure_transcript() {
    let first = seeded_failure(41);
    assert!(first.is_err());
    assert_eq!(first, seeded_failure(41));
    let different_seed = seeded_failure(42);
    assert!(different_seed.is_err());
    assert_ne!(first, different_seed);
    let Err(transcript) = first else {
        return;
    };
    let Err(different_transcript) = different_seed else {
        return;
    };
    assert_eq!(transcript.boundary, Boundary::AfterProcessEffect);
    assert_eq!(transcript.state.order.len(), MAX_TRANSCRIPT_ITEMS);
    assert_eq!(transcript.state.started.len(), 1);
    assert!(transcript.state.observed.is_empty());
    assert_ne!(transcript.state.order, different_transcript.state.order);
    println!("seed-failure-replay={transcript:?}");
    println!("seed-failure-different={different_transcript:?}");
}
