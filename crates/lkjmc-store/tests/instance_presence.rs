#[allow(dead_code)]
mod support;

use lkjmc_store::{instance, instance_presence, migrate, pool};
use serde_json::json;

#[test]
fn instance_presence_helpers_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut client = pool::connect(&url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    instance::insert(
        &mut client,
        "presence-test",
        None,
        "folia",
        "running",
        &json!({}),
    )?;
    instance_presence::upsert_heartbeat(
        &mut client,
        instance_presence::PresenceHeartbeat {
            instance_id: "presence-test",
            player_count: Some(0),
            max_players: Some(20),
            ready: true,
            implementation: Some("folia"),
        },
    )?;
    instance_presence::set_empty_since(&mut client, "presence-test")?;
    let presence =
        instance_presence::get(&mut client, "presence-test")?.ok_or("presence row missing")?;
    assert_eq!(presence.player_count, Some(0));
    assert!(presence.ready);
    assert!(presence.empty_since_age_seconds.is_some());
    instance_presence::mark_autosuspended(&mut client, "presence-test", "empty")?;
    let row = instance::get(&mut client, "presence-test")?.ok_or("instance row missing")?;
    assert_eq!(row.desired_state, "suspended");
    instance_presence::clear_autosuspended(&mut client, "presence-test")?;
    let presence =
        instance_presence::get(&mut client, "presence-test")?.ok_or("presence row missing")?;
    assert_eq!(presence.suspend_reason, None);
    Ok(())
}
