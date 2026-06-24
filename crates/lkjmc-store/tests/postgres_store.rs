use std::env;

use lkjmc_store::{audit, command, instance, jar, migrate, node, outbox, player, pool};
use serde_json::json;
use uuid::Uuid;

const TEST_SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn migrates_and_round_trips_records() -> Result<(), lkjmc_store::error::StoreError> {
    let database_url = match env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut client = pool::connect(&database_url)?;
    reset_public_schema(&mut client)?;
    let applied = migrate::apply(&mut client)?;
    assert_eq!(applied, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(migrate::apply(&mut client)?, Vec::<i32>::new());

    let node_id = Uuid::new_v4();
    node::insert(&mut client, node_id, "local", "localhost", "local-process")?;
    let stored_node = node::get(&mut client, node_id)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("node missing"))?;
    assert_eq!(stored_node.name, "local");

    let jar_id = Uuid::new_v4();
    jar::insert(&mut client, new_jar(jar_id))?;
    let stored_jar = jar::get(&mut client, jar_id)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("jar missing"))?;
    assert_eq!(stored_jar.sha256, TEST_SHA);

    instance::insert(
        &mut client,
        "hub",
        Some(node_id),
        "paper",
        "running",
        &json!({}),
    )?;
    instance::upsert_observation(
        &mut client,
        "hub",
        "process-healthy",
        Some(123),
        true,
        Some("ready"),
    )?;
    let stored_instance = instance::get(&mut client, "hub")?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("instance missing"))?;
    assert_eq!(stored_instance.kind, "paper");

    let player_id = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "PlayerOne")?;
    player::upsert_lease(&mut client, player_id, "profile", "test", 1)?;
    player::insert_snapshot(
        &mut client,
        Uuid::new_v4(),
        player_id,
        "profile",
        1,
        b"abc",
        TEST_SHA,
    )?;
    assert_eq!(
        player::get_identity_name(&mut client, player_id)?,
        Some("PlayerOne".to_string())
    );
    assert_eq!(player::snapshot_count(&mut client, player_id)?, 1);
    player::insert_session(&mut client, Uuid::new_v4(), player_id, "hub")?;
    assert_eq!(
        player::active_session_count_for_server(&mut client, "hub")?,
        1
    );

    command::insert_requested(
        &mut client,
        Uuid::new_v4(),
        "cli",
        "test",
        "status",
        &json!({}),
    )?;
    outbox::insert(
        &mut client,
        Uuid::new_v4(),
        "test.topic",
        &json!({"ok": true}),
    )?;
    audit::insert(&mut client, new_audit(Uuid::new_v4()))?;
    assert_eq!(audit::count(&mut client)?, 1);
    Ok(())
}

fn reset_public_schema(
    client: &mut postgres::Client,
) -> Result<(), lkjmc_store::error::StoreError> {
    client.batch_execute("drop schema public cascade; create schema public")?;
    Ok(())
}

fn new_jar(id: Uuid) -> jar::NewJarAsset<'static> {
    jar::NewJarAsset {
        id,
        kind: "paper",
        project: "paper",
        channel: "stable",
        name: "paper-test.jar",
        path: "/opt/lkjmc/jars/papermc/paper/paper-test.jar",
        sha256: TEST_SHA,
        size_bytes: 3,
        source: "test",
    }
}

fn new_audit(id: Uuid) -> audit::NewAuditEvent<'static> {
    audit::NewAuditEvent {
        id,
        actor_kind: "cli",
        actor_name: "test",
        action: "instance.create",
        target_kind: "instance",
        target_id: "hub",
        result: "succeeded",
    }
}
