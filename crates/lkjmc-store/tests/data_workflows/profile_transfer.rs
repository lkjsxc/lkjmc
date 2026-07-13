use lkjmc_store::{data_workflows as workflows, player};
use uuid::Uuid;

use super::helpers::{database, fail_feed, profile_json, setup_profile};

#[test]
fn profile_format_safe_complete_and_fencing_pass() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let client = db.client_mut();
    let (player_id, session, fence) = setup_profile(client)?;
    let first = player::latest_snapshot(client, player_id, "profile")?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("snapshot missing"))?;
    assert_eq!(first.sha256.len(), 64);
    let original = profile_json(1)?;
    assert!(player::write_snapshot(
        client,
        player::NewSnapshot {
            id: Uuid::new_v4(),
            player_uuid: player_id,
            scope: "profile",
            session_id: session,
            expected_session_revision: 1,
            expected_lease_fence: fence,
            expected_snapshot_revision: 0,
            correlation_id: first.correlation_id,
            source_instance: "hub",
            profile_json: &original,
        }
    )
    .is_err());
    let renewed = player::acquire_lease(client, player_id, "profile", "hub", Uuid::new_v4())?;
    assert_eq!(renewed.fence, fence + 1);
    let body = profile_json(2)?;
    let stale = player::write_snapshot(
        client,
        player::NewSnapshot {
            id: Uuid::new_v4(),
            player_uuid: player_id,
            scope: "profile",
            session_id: session,
            expected_session_revision: 2,
            expected_lease_fence: fence,
            expected_snapshot_revision: 1,
            correlation_id: Uuid::new_v4(),
            source_instance: "hub",
            profile_json: &body,
        },
    );
    assert!(stale.is_err());
    let imported = lkjmc_core::profile_validation::canonical_profile(&first.canonical_json)
        .map_err(lkjmc_store::error::StoreError::invalid_state)?;
    assert_eq!(imported.sha256, first.sha256);
    Ok(())
}

#[test]
fn transfer_crash_matrix() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let client = db.client_mut();
    let (player_id, session, fence) = setup_profile(client)?;
    fail_feed(client, "transfer")?;
    let correlation = Uuid::new_v4();
    let result = workflows::create_transfer(
        client,
        workflows::NewTransfer {
            id: Uuid::new_v4(),
            player_uuid: player_id,
            session_id: session,
            session_revision: 2,
            profile_revision: 1,
            lease_fence: fence,
            scope: "profile",
            target_server: "games",
            correlation_id: correlation,
        },
    );
    assert!(result.is_err());
    let count: i64 = client
        .query_one("select count(*) from transfer_workflows", &[])?
        .get(0);
    assert_eq!(count, 0);
    client.batch_execute(
        "drop trigger workflow_failpoint on workflow_change_feed; drop function fail_feed()",
    )?;
    let created = workflows::create_transfer(
        client,
        workflows::NewTransfer {
            id: Uuid::new_v4(),
            player_uuid: player_id,
            session_id: session,
            session_revision: 2,
            profile_revision: 1,
            lease_fence: fence,
            scope: "profile",
            target_server: "games",
            correlation_id: correlation,
        },
    )?;
    assert_eq!(
        created.state,
        lkjmc_core::data_workflow::WorkflowState::PendingSave
    );
    assert!(workflows::create_transfer(
        client,
        workflows::NewTransfer {
            id: created.id,
            player_uuid: player_id,
            session_id: session,
            session_revision: 2,
            profile_revision: 1,
            lease_fence: fence,
            scope: "other",
            target_server: "games",
            correlation_id: correlation,
        }
    )
    .is_err());
    let failed = workflows::fail(
        client,
        workflows::WorkflowTable::Transfer,
        created.id,
        1,
        fence,
        "timeout",
    )?;
    assert_eq!(failed.revision, 2);
    assert!(
        workflows::fail(
            client,
            workflows::WorkflowTable::Transfer,
            created.id,
            1,
            fence,
            "timeout"
        )?
        .replay
    );
    assert!(workflows::fail(
        client,
        workflows::WorkflowTable::Transfer,
        created.id,
        1,
        fence,
        "changed"
    )
    .is_err());
    assert!(workflows::fail(
        client,
        workflows::WorkflowTable::Transfer,
        created.id,
        1,
        fence + 1,
        "timeout"
    )
    .is_err());
    Ok(())
}
