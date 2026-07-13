#[allow(dead_code)]
mod support;

use lkjmc_store::{instance, instance_presence, migrate};
use serde_json::json;

#[test]
fn instance_presence_helpers_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    instance::insert(
        client,
        "presence-test",
        None,
        "folia",
        "running",
        &json!({}),
    )?;
    instance_presence::upsert_heartbeat(
        client,
        instance_presence::PresenceHeartbeat {
            instance_id: "presence-test",
            player_count: Some(0),
            max_players: Some(20),
            ready: true,
            implementation: Some("folia"),
        },
    )?;
    instance_presence::set_empty_since(client, "presence-test")?;
    let presence =
        instance_presence::get(client, "presence-test")?.ok_or("presence row missing")?;
    assert_eq!(presence.player_count, Some(0));
    assert!(presence.ready);
    assert!(presence.empty_since_age_seconds.is_some());
    instance_presence::mark_autosuspended(client, "presence-test", "empty")?;
    let row = instance::get(client, "presence-test")?.ok_or("instance row missing")?;
    assert_eq!(row.desired_state, "suspended");
    instance_presence::clear_autosuspended(client, "presence-test")?;
    let presence =
        instance_presence::get(client, "presence-test")?.ok_or("presence row missing")?;
    assert_eq!(presence.suspend_reason, None);
    Ok(())
}
