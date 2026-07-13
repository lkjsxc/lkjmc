#[allow(dead_code)]
mod support;

use lkjmc_store::{instance, migrate, player, wake_join};
use serde_json::json;
use uuid::Uuid;

#[test]
fn wake_join_queue_records_state_transitions() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let player_uuid = Uuid::new_v4();
    player::insert_identity(client, player_uuid, "PlayerOne")?;
    instance::insert(client, "sleepy", None, "folia", "suspended", &json!({}))?;
    let id = Uuid::new_v4();
    let created = wake_join::create_or_live(
        client,
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
    wake_join::mark_starting(client, id)?;
    wake_join::mark_ready(client, id, "sleepy")?;
    let stored = wake_join::get(client, id)?.ok_or("wake row missing")?;
    assert_eq!(stored.state, "ready");
    assert_eq!(stored.target_server.as_deref(), Some("sleepy"));
    Ok(())
}
