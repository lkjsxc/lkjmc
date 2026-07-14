use uuid::Uuid;

use super::fixture::Fixture;
use super::request;
use crate::runtime::RuntimeGoal;

#[test]
fn fault_before_effect_is_failed_and_queryable() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    let old = fixture.seed_attempt("fault-before-effect", "none")?;
    reapply(&fixture, "retry-before-effect")?;
    assert_old(&mut fixture, old, "failed")?;
    assert_retry(&mut fixture, "retry-before-effect")?;
    assert_cleanup(&mut fixture)
}

#[test]
fn fault_after_config_render_has_no_rollback_claim() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    fixture.render_proxy()?;
    let old = fixture.seed_attempt("fault-after-render", "configuration")?;
    assert!(fixture.tracked_pids()?.is_empty());
    reapply(&fixture, "retry-after-render")?;
    let attempt = assert_old(&mut fixture, old, "failed")?;
    assert_eq!(attempt.observation["rollbackClaimed"], false);
    assert_retry(&mut fixture, "retry-after-render")?;
    assert_cleanup(&mut fixture)
}

#[test]
fn fault_after_child_start_adopts_fenced_child() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    let old = fixture.seed_attempt("fault-after-child", "runtime")?;
    let pid = fixture
        .start_proxy()?
        .pid()
        .ok_or("started proxy identity missing")?;
    let restarted = fixture.restarted_state()?;
    reapply_state(&restarted, "retry-after-child")?;
    let attempt = assert_old(&mut fixture, old, "unknown")?;
    assert_eq!(
        attempt.observation["resources"]["proxy"]["reconciled"]["identity"]["pid"],
        pid
    );
    assert!(crate::runtime::process::group_exists(pid));
    crate::support::instance_helpers::stop_runtime(&restarted, "proxy")?;
    assert_retry(&mut fixture, "retry-after-child")?;
    assert_cleanup(&mut fixture)
}

#[test]
fn fault_after_observation_repeats_real_observation() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    let old = fixture.seed_attempt("fault-after-observation", "runtime")?;
    let pid = fixture
        .start_proxy()?
        .pid()
        .ok_or("started proxy identity missing")?;
    crate::runtime::reconcile::reconcile(
        &fixture.state,
        "proxy",
        RuntimeGoal::Observe,
        Uuid::new_v4(),
    )?;
    lkjmc_store::network_intent::mark_effect_phase(
        fixture.database.client_mut(),
        old,
        "observation",
    )
    .map_err(|error| error.to_string())?;
    reapply(&fixture, "retry-after-observation")?;
    let attempt = assert_old(&mut fixture, old, "unknown")?;
    assert_eq!(
        attempt.observation["resources"]["proxy"]["observed"]["identity"]["pid"],
        pid
    );
    assert_retry(&mut fixture, "retry-after-observation")?;
    assert_cleanup(&mut fixture)
}

#[test]
fn daemon_restart_stops_child_when_intent_changes() -> Result<(), String> {
    let Some(mut fixture) = Fixture::new("python3")? else {
        return Ok(());
    };
    let old = fixture.seed_attempt("fault-daemon-restart", "runtime")?;
    let pid = fixture
        .start_proxy()?
        .pid()
        .ok_or("started proxy identity missing")?;
    fixture.set_proxy_stopped()?;
    let restarted = fixture.restarted_state()?;
    let response = super::super::apply(&restarted, request("retry-daemon-restart")?);
    if !response.ok {
        return Err(format!("restart recovery failed: {:?}", response.error));
    }
    let attempt = assert_old(&mut fixture, old, "unknown")?;
    assert!(
        attempt.observation["resources"]["proxy"]["reconciled"]["observedState"]
            .as_str()
            .is_some_and(|value| value.contains("absent"))
    );
    assert!(!crate::runtime::process::group_exists(pid));
    assert_retry(&mut fixture, "retry-daemon-restart")?;
    let _ = restarted.shutdown_runtime();
    assert_cleanup(&mut fixture)
}

fn reapply(fixture: &Fixture, correlation: &str) -> Result<(), String> {
    reapply_state(&fixture.state, correlation)
}

fn reapply_state(state: &crate::app::AppState, correlation: &str) -> Result<(), String> {
    let response = super::super::apply(state, request(correlation)?);
    if response.ok {
        Ok(())
    } else {
        Err(format!("reapply failed: {:?}", response.error))
    }
}

fn assert_old(
    fixture: &mut Fixture,
    id: Uuid,
    outcome: &str,
) -> Result<lkjmc_store::network_intent::ApplyAttempt, String> {
    let attempt = fixture.attempt(id)?;
    assert_eq!(attempt.outcome, outcome);
    assert_ne!(attempt.outcome, "observed");
    assert_eq!(attempt.observation["recoveryComplete"], true);
    Ok(attempt)
}

fn assert_retry(fixture: &mut Fixture, correlation: &str) -> Result<(), String> {
    let attempts = fixture.attempts_for(correlation)?;
    assert_eq!(attempts.len(), 1);
    assert!(matches!(attempts[0].outcome.as_str(), "observed" | "no-op"));
    Ok(())
}

fn assert_cleanup(fixture: &mut Fixture) -> Result<(), String> {
    let pids = fixture.tracked_pids()?;
    fixture.cleanup();
    assert!(pids
        .into_iter()
        .all(|pid| !crate::runtime::process::group_exists(pid)));
    Ok(())
}
