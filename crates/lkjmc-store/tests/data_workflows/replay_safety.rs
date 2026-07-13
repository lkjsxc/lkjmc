use std::sync::{Arc, Barrier};
use std::thread;

use lkjmc_store::{data_workflows as workflows, player, pool};
use uuid::Uuid;

use super::helpers::{database, profile_json, setup_profile};

#[test]
fn profile_replay_binds_all_revisions() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let client = db.client_mut();
    let (player_id, session, fence) = setup_profile(client)?;
    let id = Uuid::new_v4();
    let correlation = Uuid::new_v4();
    let body = profile_json(2)?;
    let write = |session_revision, snapshot_revision, correlation| player::NewSnapshot {
        id,
        player_uuid: player_id,
        scope: "profile",
        session_id: session,
        expected_session_revision: session_revision,
        expected_lease_fence: fence,
        expected_snapshot_revision: snapshot_revision,
        correlation_id: correlation,
        source_instance: "hub",
        profile_json: &body,
    };
    assert!(player::write_snapshot(client, write(1, 1, Uuid::new_v4())).is_err());
    assert!(player::write_snapshot(client, write(2, 0, Uuid::new_v4())).is_err());
    let saved = player::write_snapshot(client, write(2, 1, correlation))?;
    assert_eq!(saved.revision, 2);
    assert!(player::write_snapshot(client, write(2, 1, correlation))?.replay);
    assert!(player::write_snapshot(client, write(2, 1, Uuid::new_v4())).is_err());
    Ok(())
}

#[test]
fn transfer_replay_staleness_and_failure_race() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut db) = database()? else {
        return Ok(());
    };
    let url = db.url().to_string();
    let client = db.client_mut();
    let (player, session, fence) = setup_profile(client)?;
    let id = Uuid::new_v4();
    let correlation = Uuid::new_v4();
    let create = |id, correlation, session_revision, profile_revision| workflows::NewTransfer {
        id,
        player_uuid: player,
        session_id: session,
        session_revision,
        profile_revision,
        lease_fence: fence,
        scope: "profile",
        target_server: "games",
        correlation_id: correlation,
    };
    let created = workflows::create_transfer(client, create(id, correlation, 2, 1))?;
    assert!(workflows::create_transfer(client, create(id, correlation, 2, 1))?.replay);
    assert!(workflows::create_transfer(client, create(Uuid::new_v4(), correlation, 2, 1)).is_err());
    assert!(
        workflows::create_transfer(client, create(Uuid::new_v4(), Uuid::new_v4(), 1, 1)).is_err()
    );
    assert!(
        workflows::create_transfer(client, create(Uuid::new_v4(), Uuid::new_v4(), 2, 2)).is_err()
    );

    let workflow_id = created.id;
    let barrier = Arc::new(Barrier::new(2));
    let workers = (0..2)
        .map(|_| {
            let url = url.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                let mut client = pool::connect(&url)?;
                barrier.wait();
                workflows::fail(
                    &mut client,
                    workflows::WorkflowTable::Transfer,
                    workflow_id,
                    1,
                    fence,
                    "timeout",
                )
                .map(|record| record.replay)
            })
        })
        .collect::<Vec<_>>();
    let mut outcomes = Vec::new();
    for worker in workers {
        let outcome = worker.join().map_err(|_| {
            lkjmc_store::error::StoreError::invalid_state("failure worker panicked")
        })??;
        outcomes.push(outcome);
    }
    outcomes.sort_unstable();
    assert_eq!(outcomes, vec![false, true]);
    assert!(workflows::fail(
        client,
        workflows::WorkflowTable::Transfer,
        created.id,
        1,
        fence,
        "different"
    )
    .is_err());
    Ok(())
}
