#[allow(dead_code)]
mod support;

use lkjmc_store::{instance, migrate, player, pool, wake_join};
use serde_json::json;
use uuid::Uuid;

#[test]
fn wake_join_queue_records_state_transitions() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(());
    };
    let mut client = pool::connect(&url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let player_uuid = Uuid::new_v4();
    player::insert_identity(&mut client, player_uuid, "PlayerOne")?;
    instance::insert(
        &mut client,
        "sleepy",
        None,
        "folia",
        "suspended",
        &json!({}),
    )?;
    let id = Uuid::new_v4();
    let created = wake_join::create_or_live(
        &mut client,
        wake_join::NewWakeJoin {
            id,
            player_uuid,
            player_name: "PlayerOne",
            target_instance_id: "sleepy",
            requested_by_kind: "velocity-plugin",
            requested_by_name: "velocity",
            expires_in_seconds: 30,
            correlation_id: "test-correlation",
            metadata: json!({}),
        },
    )?;
    assert_eq!(created.state, "queued");
    wake_join::mark_starting(&mut client, id)?;
    wake_join::mark_ready(&mut client, id, "sleepy")?;
    let stored = wake_join::get(&mut client, id)?.ok_or("wake row missing")?;
    assert_eq!(stored.state, "ready");
    assert_eq!(stored.target_server.as_deref(), Some("sleepy"));
    Ok(())
}
