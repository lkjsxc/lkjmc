mod support;

use lkjmc_store::{audit, command, instance, jar, migrate, node, outbox, player, pool};
use serde_json::json;
use std::env;
use support::{new_audit, new_jar, reset_public_schema, TEST_SHA};
use uuid::Uuid;
#[test]
fn migrates_and_round_trips_records() -> Result<(), lkjmc_store::error::StoreError> {
    let database_url = match env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut client = pool::connect(&database_url)?;
    reset_public_schema(&mut client)?;
    let applied = migrate::apply(&mut client)?;
    assert_eq!(applied, (1..=15).collect::<Vec<_>>());
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
    assert_eq!(
        instance::allocate_port(&mut client, "hub", "server", 25565, 25565)?,
        25565
    );
    let player_id = Uuid::new_v4();
    player::insert_identity(&mut client, player_id, "PlayerOne")?;
    player::upsert_lease(&mut client, player_id, "profile", "test", 1)?;
    let lease_revision = player::acquire_lease(&mut client, player_id, "profile", "test")?;
    assert_eq!(lease_revision, 1);
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
    assert_eq!(
        player::latest_snapshot(&mut client, player_id, "profile")?
            .map(|snapshot| snapshot.revision),
        Some(1)
    );
    lkjmc_store::player_settings::set_language(&mut client, player_id, "ja")?;
    lkjmc_store::player_settings::set_hud(&mut client, player_id, true)?;
    assert_eq!(
        lkjmc_store::player_settings::language(&mut client, player_id)?,
        Some("ja".to_string())
    );
    lkjmc_store::points::ensure_account(&mut client, player_id)?;
    assert_eq!(
        lkjmc_store::player_settings::hud_enabled(&mut client, player_id)?,
        Some(true)
    );
    assert_eq!(lkjmc_store::points::balance(&mut client, player_id)?, 0);
    assert!(lkjmc_store::daily::claim(&mut client, player_id, 3)?);
    lkjmc_store::points::grant(&mut client, player_id, 10, "test")?;
    lkjmc_store::shop::upsert_item(&mut client, "apple", "shop.apple", 5)?;
    let item = lkjmc_store::shop::get_item(&mut client, "apple")?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("missing item"))?;
    assert!(lkjmc_store::points::spend(
        &mut client,
        player_id,
        item.price_points,
        "shop"
    )?);
    lkjmc_store::shop::record_purchase(&mut client, player_id, &item)?;
    lkjmc_store::homes::upsert(
        &mut client,
        Uuid::new_v4(),
        player_id,
        "base",
        "hub",
        serde_json::json!({"x": 1.0}),
    )?;
    assert_eq!(
        lkjmc_store::homes::get(&mut client, player_id, "base")?.map(|home| home.server_id),
        Some("hub".to_string())
    );
    lkjmc_store::warps::upsert(&mut client, "spawn", "hub", serde_json::json!({"x": 2.0}))?;
    assert_eq!(
        lkjmc_store::warps::get(&mut client, "spawn")?.map(|warp| warp.server_id),
        Some("hub".to_string())
    );
    let target_location = serde_json::json!({"world": "world", "x": 3.0});
    lkjmc_store::teleport::request(&mut client, player_id, "minigame", "hub", target_location)?;
    assert!(lkjmc_store::teleport::take(&mut client, player_id, "minigame")?.is_some());
    let party_id = Uuid::new_v4();
    lkjmc_store::party::create(&mut client, party_id, player_id, "alpha")?;
    assert_eq!(
        lkjmc_store::party::current(&mut client, player_id)?.and_then(|party| party.name),
        Some("alpha".to_string())
    );
    let invitee = Uuid::new_v4();
    player::insert_identity(&mut client, invitee, "invitee")?;
    lkjmc_store::party::invite(&mut client, Uuid::new_v4(), party_id, player_id, invitee)?;
    let invite = lkjmc_store::party::pending_invite(&mut client, invitee)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("missing invite"))?;
    lkjmc_store::party::accept(&mut client, invite.id, invite.party_id, invitee)?;
    assert!(lkjmc_store::party::current(&mut client, invitee)?.is_some());
    let mail_id = Uuid::new_v4();
    lkjmc_store::mail::send(
        &mut client,
        mail_id,
        invitee,
        player_id,
        "PlayerOne",
        "hello",
    )?;
    assert!(lkjmc_store::mail::read(&mut client, invitee, mail_id)?.is_some());
    lkjmc_store::reports::create(
        &mut client,
        Uuid::new_v4(),
        player_id,
        invitee,
        "hub",
        "test",
    )?;
    assert_eq!(lkjmc_store::reports::open(&mut client, 10)?.len(), 1);
    lkjmc_store::moderation::ban(
        &mut client,
        Uuid::new_v4(),
        invitee,
        "invitee",
        "op",
        "test",
    )?;
    assert!(lkjmc_store::moderation::active_ban(&mut client, invitee)?.is_some());
    lkjmc_store::achievement::grant(
        &mut client,
        player_id,
        "first-login",
        "achievement.first-login",
    )?;
    assert_eq!(
        lkjmc_store::achievement::list_claimed(&mut client, player_id)?
            .first()
            .map(|achievement| achievement.id.clone()),
        Some("first-login".to_string())
    );
    lkjmc_store::player_session::insert(&mut client, Uuid::new_v4(), player_id, "hub")?;
    assert_eq!(
        lkjmc_store::player_session::active_count_for_server(&mut client, "hub")?,
        1
    );
    lkjmc_store::player_session::leave(&mut client, player_id, "hub")?;
    assert_eq!(
        lkjmc_store::player_session::active_count_for_server(&mut client, "hub")?,
        0
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
