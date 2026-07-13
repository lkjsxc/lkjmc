#[allow(dead_code)]
mod support;

use lkjmc_store::runtime_adoption::{allocate, history_count, mark_effect, observe, owns, pending};
use serde_json::json;
use uuid::Uuid;

fn with_database(
    test: impl FnOnce(&mut postgres::Client) -> Result<(), String>,
) -> Result<(), String> {
    let Some(mut database) = support::database().map_err(|error| error.to_string())? else {
        return Ok(());
    };
    lkjmc_store::migrate::apply(database.client_mut()).map_err(|error| error.to_string())?;
    lkjmc_store::instance::insert(
        database.client_mut(),
        "runtime-test",
        None,
        "paper",
        "stopped",
        &json!({"template":"paper"}),
    )
    .map_err(|error| error.to_string())?;
    test(database.client_mut())
}

#[test]
fn reconcile_idempotent() -> Result<(), String> {
    with_database(|client| {
        let correlation = Uuid::new_v4();
        let first = allocate(
            client,
            "runtime-test",
            "start",
            &json!({"desired":"running"}),
            correlation,
        )
        .map_err(|error| error.to_string())?;
        let replay = allocate(
            client,
            "runtime-test",
            "start",
            &json!({"desired":"running"}),
            correlation,
        )
        .map_err(|error| error.to_string())?;
        assert!(replay.replay);
        assert_eq!(first.id, replay.id);
        assert_eq!(first.fence, replay.fence);
        assert!(owns(client, &first).map_err(|error| error.to_string())?);
        assert!(observe(
            client,
            &first,
            &json!({"state":"running"}),
            "succeeded",
            None
        )
        .map_err(|error| error.to_string())?);
        assert!(history_count(client, first.id).map_err(|error| error.to_string())? >= 3);
        Ok(())
    })
}

#[test]
fn crash_after_intent_before_effect_is_recoverable() -> Result<(), String> {
    with_database(|client| {
        let operation = allocate(
            client,
            "runtime-test",
            "start",
            &json!({"desired":"running"}),
            Uuid::new_v4(),
        )
        .map_err(|error| error.to_string())?;
        let recovery = pending(client, "runtime-test")
            .map_err(|error| error.to_string())?
            .ok_or("pending operation missing")?;
        assert_eq!(recovery.operation.id, operation.id);
        assert!(!recovery.effect_started);
        assert_eq!(
            history_count(client, operation.id).map_err(|error| error.to_string())?,
            1
        );
        Ok(())
    })
}

#[test]
fn effect_before_observation_remains_pending() -> Result<(), String> {
    with_database(|client| {
        let operation = allocate(
            client,
            "runtime-test",
            "start",
            &json!({"desired":"running"}),
            Uuid::new_v4(),
        )
        .map_err(|error| error.to_string())?;
        assert!(mark_effect(client, &operation).map_err(|error| error.to_string())?);
        let recovery = pending(client, "runtime-test")
            .map_err(|error| error.to_string())?
            .ok_or("pending operation missing")?;
        assert!(recovery.effect_started);
        assert_eq!(recovery.operation.id, operation.id);
        Ok(())
    })
}

#[test]
fn effect_crash_recovery() -> Result<(), String> {
    with_database(|client| {
        let stale = allocate(
            client,
            "runtime-test",
            "start",
            &json!({"desired":"running"}),
            Uuid::new_v4(),
        )
        .map_err(|error| error.to_string())?;
        assert!(owns(client, &stale).map_err(|error| error.to_string())?);
        let recovery = allocate(
            client,
            "runtime-test",
            "observe",
            &json!({"recover":true}),
            Uuid::new_v4(),
        )
        .map_err(|error| error.to_string())?;
        assert!(recovery.fence > stale.fence);
        assert!(!observe(
            client,
            &stale,
            &json!({"state":"running"}),
            "succeeded",
            Some("late result"),
        )
        .map_err(|error| error.to_string())?);
        assert!(owns(client, &recovery).map_err(|error| error.to_string())?);
        assert!(observe(
            client,
            &recovery,
            &json!({"state":"unknown"}),
            "unknown",
            Some("crash window observed"),
        )
        .map_err(|error| error.to_string())?);
        Ok(())
    })
}
