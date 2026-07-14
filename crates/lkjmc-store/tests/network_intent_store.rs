#[allow(dead_code)]
mod support;

use lkjmc_store::network_intent;
use serde_json::json;

#[test]
fn desired_intent_and_partial_history_are_durable() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    lkjmc_store::migrate::apply(client)?;
    let digest = "780ecd0728ca321d6421db8597a09b6d34c6e4f6dd622de86cad8412c6a12685";
    let (desired, failed) = network_intent::record_desired_with_attempt(
        client,
        7,
        digest,
        &json!({"revision": 7}),
        "network-test-1",
    )?;
    let repeated = network_intent::record_desired(
        client,
        7,
        digest,
        &json!({"revision": 7}),
        "network-test-1",
    )?;
    assert_eq!(desired.revision, repeated.revision);
    network_intent::mark_applying(client, failed.id)?;
    network_intent::finish_attempt(
        client,
        failed.id,
        "failed",
        Some("readiness timeout"),
        &json!({"hub": "unready"}),
    )?;
    let unknown = network_intent::create_attempt(client, desired.revision, "network-test-unknown")?;
    network_intent::mark_applying(client, unknown.id)?;
    network_intent::mark_effect_phase(client, unknown.id, "runtime")?;
    network_intent::finish_attempt(
        client,
        unknown.id,
        "unknown",
        Some("child may have started"),
        &json!({"recoveryComplete": false}),
    )?;
    assert_eq!(network_intent::recovery_candidates(client)?.len(), 1);
    network_intent::complete_unknown(
        client,
        unknown.id,
        &json!({"recoveryComplete": true, "hub": "ready"}),
    )?;
    let repair = network_intent::create_attempt(client, desired.revision, "network-test-repair")?;
    network_intent::finish_attempt(
        client,
        repair.id,
        "observed",
        None,
        &json!({"hub": "ready"}),
    )?;
    let history = network_intent::attempts_for_revision(client, desired.revision)?;
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].outcome, "failed");
    assert_eq!(history[1].outcome, "unknown");
    assert_eq!(history[1].observation["recoveryComplete"], true);
    assert_eq!(history[2].outcome, "observed");
    assert_eq!(
        network_intent::latest_desired(client)?.map(|item| item.revision),
        Some(desired.revision)
    );
    Ok(())
}

#[test]
fn intent_and_attempt_rollback_together() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    lkjmc_store::migrate::apply(client)?;
    client.batch_execute(
        "create function reject_network_attempt() returns trigger language plpgsql as $$
         begin raise exception 'injected attempt failure'; end $$;
         create trigger reject_network_attempt before insert on network_apply_attempts
         for each row when (new.correlation = 'network-test-rollback')
         execute function reject_network_attempt();",
    )?;
    let digest = "d97b98f646f1e38cf9c46fa8811a9f9bad8b0d96ddbc482769e7b397ba830897";
    assert!(network_intent::record_desired_with_attempt(
        client,
        9,
        digest,
        &json!({"revision": 9}),
        "network-test-rollback",
    )
    .is_err());
    let count: i64 = client
        .query_one(
            "select count(*) from network_intents where correlation = $1",
            &[&"network-test-rollback"],
        )?
        .get(0);
    assert_eq!(count, 0);
    Ok(())
}

#[test]
fn correlation_cannot_change_owned_intent() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    lkjmc_store::migrate::apply(client)?;
    let first = "780ecd0728ca321d6421db8597a09b6d34c6e4f6dd622de86cad8412c6a12685";
    let second = "3f3d4f4cdaff94f0089cd7fe6f78acb7475c8ccdcfef4ae4f462b6549f3da747";
    network_intent::record_desired(client, 1, first, &json!({"revision": 1}), "network-test-2")?;
    assert!(network_intent::record_desired(
        client,
        2,
        second,
        &json!({"revision": 2}),
        "network-test-2"
    )
    .is_err());
    Ok(())
}
