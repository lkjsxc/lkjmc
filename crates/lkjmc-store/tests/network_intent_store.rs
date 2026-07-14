mod support;

use lkjmc_store::network_intent;
use serde_json::json;

#[test]
fn desired_intent_and_partial_history_are_durable() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else { return Ok(()); };
    let client = database.client_mut();
    lkjmc_store::migrate::apply(client)?;
    let digest = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let desired = network_intent::record_desired(client, 7, digest, &json!({"revision": 7}), "network-test-1")?;
    let repeated = network_intent::record_desired(client, 7, digest, &json!({"revision": 7}), "network-test-1")?;
    assert_eq!(desired.revision, repeated.revision);
    let failed = network_intent::create_attempt(client, desired.revision, "network-test-1")?;
    network_intent::mark_applying(client, failed.id)?;
    network_intent::finish_attempt(client, failed.id, "failed", Some("readiness timeout"), &json!({"hub": "unready"}))?;
    let repair = network_intent::create_attempt(client, desired.revision, "network-test-repair")?;
    network_intent::finish_attempt(client, repair.id, "observed", None, &json!({"hub": "ready"}))?;
    let history = network_intent::attempts_for_revision(client, desired.revision)?;
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].outcome, "failed");
    assert_eq!(history[1].outcome, "observed");
    assert_eq!(network_intent::latest_desired(client)?.map(|item| item.revision), Some(desired.revision));
    Ok(())
}

#[test]
fn correlation_cannot_change_owned_intent() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else { return Ok(()); };
    let client = database.client_mut();
    lkjmc_store::migrate::apply(client)?;
    let first = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let second = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    network_intent::record_desired(client, 1, first, &json!({"revision": 1}), "network-test-2")?;
    assert!(network_intent::record_desired(client, 2, second, &json!({"revision": 2}), "network-test-2").is_err());
    Ok(())
}
