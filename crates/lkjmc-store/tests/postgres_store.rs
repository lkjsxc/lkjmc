#[allow(dead_code)]
mod support;

use lkjmc_store::{audit, instance, jar, migrate, node, player};
use serde_json::json;
use support::{new_audit, new_jar, TEST_SHA};
use uuid::Uuid;
#[test]
fn migrates_and_round_trips_records() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    let applied = migrate::apply(client)?;
    assert_eq!(applied, migrate::embedded_versions());
    assert_eq!(migrate::apply(client)?, Vec::<i32>::new());
    let node_id = Uuid::new_v4();
    node::insert(client, node_id, "local", "localhost", "local-process")?;
    let stored_node = node::get(client, node_id)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("node missing"))?;
    assert_eq!(stored_node.name, "local");
    let jar_id = Uuid::new_v4();
    jar::insert(client, new_jar(jar_id))?;
    let stored_jar = jar::get(client, jar_id)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("jar missing"))?;
    assert_eq!(stored_jar.sha256, TEST_SHA);
    instance::insert(client, "hub", Some(node_id), "paper", "running", &json!({}))?;
    instance::upsert_observation(
        client,
        "hub",
        "process-healthy",
        Some(123),
        true,
        Some("ready"),
    )?;
    let stored_instance = instance::get(client, "hub")?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("instance missing"))?;
    assert_eq!(stored_instance.kind, "paper");
    let allocated = instance::allocate_port(client, "hub", "server", 25565, 25565)?;
    assert_eq!(allocated, 25565);
    let player_id = Uuid::new_v4();
    player::insert_identity(client, player_id, "PlayerOne")?;
    let session_id = Uuid::new_v4();
    lkjmc_store::player_session::insert(client, session_id, player_id, "test")?;
    let lease = player::acquire_lease(client, player_id, "profile", "test", Uuid::new_v4())?;
    assert_eq!(lease.fence, 1);
    let profile = serde_json::to_vec(&json!({
        "schema":"lkjmc-profile-one","inventory":[],"armor":[],"offhand":null,
        "selectedHotbarSlot":0,"enderChest":[],
        "experience":{"progress":0.0,"level":0,"total":0},
        "vitals":{"health":20.0,"food":20,"saturation":5.0,"air":300},
        "potionEffects":[],"gameMode":null,"pluginData":[],"homes":[],"warps":[],
        "points":0,"achievements":[],
        "settings":{"menuEnabled":true,"hudEnabled":true,"tipsEnabled":true,"privacy":"private"},
        "language":"en"
    }))
    .map_err(|error| {
        lkjmc_store::error::StoreError::invalid_state(format!(
            "profile fixture serialization failed: {error}"
        ))
    })?;
    player::write_snapshot(
        client,
        player::NewSnapshot {
            id: Uuid::new_v4(),
            player_uuid: player_id,
            scope: "profile",
            session_id,
            expected_session_revision: 1,
            expected_lease_fence: lease.fence,
            expected_snapshot_revision: 0,
            correlation_id: Uuid::new_v4(),
            source_instance: "test",
            profile_json: &profile,
        },
    )?;
    assert_eq!(
        player::get_identity_name(client, player_id)?,
        Some("PlayerOne".to_string())
    );
    assert_eq!(player::snapshot_count(client, player_id)?, 1);
    assert_eq!(
        player::latest_snapshot(client, player_id, "profile")?.map(|snapshot| snapshot.revision),
        Some(1)
    );
    lkjmc_store::player_settings::set_language(client, player_id, "ja")?;
    lkjmc_store::player_settings::set_hud(client, player_id, true)?;
    assert_eq!(
        lkjmc_store::player_settings::language(client, player_id)?,
        Some("ja".to_string())
    );
    lkjmc_store::points::ensure_account(client, player_id)?;
    assert_eq!(
        lkjmc_store::player_settings::hud_enabled(client, player_id)?,
        Some(true)
    );
    assert_eq!(lkjmc_store::points::balance(client, player_id)?, 0);
    assert!(lkjmc_store::daily::claim(client, player_id, 3)?);
    lkjmc_store::points::grant(client, player_id, 10, "test")?;
    lkjmc_store::shop::upsert_item(client, "apple", "shop.apple", 5)?;
    let item = lkjmc_store::shop::get_item(client, "apple")?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("missing item"))?;
    assert!(lkjmc_store::points::spend(
        client,
        player_id,
        item.price_points,
        "shop"
    )?);
    lkjmc_store::shop::record_purchase(client, player_id, &item)?;
    lkjmc_store::homes::upsert(
        client,
        Uuid::new_v4(),
        player_id,
        "base",
        "hub",
        serde_json::json!({"x": 1.0}),
    )?;
    assert_eq!(
        lkjmc_store::homes::get(client, player_id, "base")?.map(|home| home.server_id),
        Some("hub".to_string())
    );
    lkjmc_store::warps::upsert(client, "spawn", "hub", serde_json::json!({"x": 2.0}))?;
    assert_eq!(
        lkjmc_store::warps::get(client, "spawn")?.map(|warp| warp.server_id),
        Some("hub".to_string())
    );
    let target_location = serde_json::json!({"world": "world", "x": 3.0});
    lkjmc_store::teleport::request(client, player_id, "minigame", "hub", target_location)?;
    assert!(lkjmc_store::teleport::take(client, player_id, "minigame")?.is_some());
    let party_id = Uuid::new_v4();
    lkjmc_store::party::create(client, party_id, player_id, "alpha")?;
    assert_eq!(
        lkjmc_store::party::current(client, player_id)?.and_then(|party| party.name),
        Some("alpha".to_string())
    );
    let invitee = Uuid::new_v4();
    player::insert_identity(client, invitee, "invitee")?;
    lkjmc_store::party::invite(client, Uuid::new_v4(), party_id, player_id, invitee)?;
    let invite = lkjmc_store::party::pending_invite(client, invitee)?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("missing invite"))?;
    lkjmc_store::party::accept(client, invite.id, invite.party_id, invitee)?;
    assert!(lkjmc_store::party::current(client, invitee)?.is_some());
    let members = lkjmc_store::party::members(client, party_id)?;
    assert_eq!(members.len(), 2);
    let mail_id = Uuid::new_v4();
    lkjmc_store::mail::send(client, mail_id, invitee, player_id, "PlayerOne", "hello")?;
    assert!(lkjmc_store::mail::read(client, invitee, mail_id)?.is_some());
    lkjmc_store::reports::create(client, Uuid::new_v4(), player_id, invitee, "hub", "test")?;
    assert_eq!(lkjmc_store::reports::open(client, 10)?.len(), 1);
    lkjmc_store::moderation::ban(client, Uuid::new_v4(), invitee, "invitee", "op", "test")?;
    assert!(lkjmc_store::moderation::active_ban(client, invitee)?.is_some());
    lkjmc_store::achievement::grant(client, player_id, "first-login", "achievement.first-login")?;
    assert_eq!(
        lkjmc_store::achievement::list_claimed(client, player_id)?
            .first()
            .map(|achievement| achievement.id.clone()),
        Some("first-login".to_string())
    );
    assert_eq!(
        lkjmc_store::player_session::active_count_for_server(client, "test")?,
        1
    );
    lkjmc_store::player_session::leave(client, player_id, "test")?;
    assert_eq!(
        lkjmc_store::player_session::active_count_for_server(client, "test")?,
        0
    );
    audit::insert(client, new_audit(Uuid::new_v4()))?;
    assert_eq!(audit::count(client)?, 1);
    Ok(())
}
