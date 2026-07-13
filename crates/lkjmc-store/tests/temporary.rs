#[allow(dead_code)]
mod support;

use lkjmc_store::{instance, migrate, temporary};
use serde_json::json;
use uuid::Uuid;

#[test]
fn temporary_adventure_helpers_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    instance::insert(client, "temp-end-1", None, "folia", "stopped", &json!({}))?;
    let buyer_uuid = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let mut tx = client.transaction()?;
    let temporary = temporary::insert_instance(
        &mut tx,
        temporary::NewTemporaryInstance {
            instance_id: "temp-end-1",
            owner_kind: "adventure",
            owner_id: &session_id.to_string(),
            visibility: "hidden",
            world_path: "/srv/lkjmc/worlds/temp-end-1",
            server_port: 30001,
            max_lifetime_seconds: 3600,
            retention_seconds: 600,
            cleanup_policy: "delete",
            lifecycle_state: "created",
            start_deadline_seconds: 120,
            metadata: json!({"adventure":"end-expedition"}),
        },
    )?;
    let session = temporary::insert_session(
        &mut tx,
        temporary::NewAdventureSession {
            id: session_id,
            adventure_kind: "end-expedition",
            buyer_uuid,
            buyer_name: "PlayerOne",
            temporary_instance_id: "temp-end-1",
            points_cost: 100,
            points_ledger_id: None,
            state: "pending",
            start_deadline_seconds: 120,
            stop_deadline_seconds: 3600,
            metadata: json!({}),
        },
    )?;
    temporary::add_participant(
        &mut tx,
        temporary::NewAdventureParticipant {
            session_id,
            player_uuid: buyer_uuid,
            player_name: "PlayerOne",
            role: "buyer",
            state: "queued",
            metadata: json!({}),
        },
    )?;
    tx.commit()?;
    assert_eq!(temporary.instance_id, "temp-end-1");
    assert_eq!(session.state, "pending");
    temporary::update_instance_state(client, "temp-end-1", "ready", None)?;
    temporary::record_cleanup_event(
        client,
        Uuid::new_v4(),
        "temp-end-1",
        "cleanup-attempt",
        "succeeded",
        None,
    )?;
    let intent_id = Uuid::new_v4();
    let intent = temporary::create_intent(
        client,
        temporary::NewTransferIntent {
            id: intent_id,
            temporary_instance_id: "temp-end-1",
            player_uuid: buyer_uuid,
            player_name: "PlayerOne",
            expires_in_seconds: 30,
            metadata: json!({}),
        },
    )?;
    client.execute(
        "update temporary_instances set expires_at = now() - interval '1 second',
         retain_until = now() - interval '1 second' where instance_id = $1",
        &[&"temp-end-1"],
    )?;
    let loaded = temporary::get_instance(client, "temp-end-1")?.ok_or("missing temp")?;
    let loaded_session = temporary::get_session(client, session_id)?.ok_or("missing session")?;
    let candidates = temporary::cleanup_candidates(client, 10)?;
    assert_eq!(loaded.lifecycle_state, "ready");
    assert_eq!(loaded_session.temporary_instance_id, "temp-end-1");
    assert_eq!(intent.temporary_instance_id, "temp-end-1");
    assert_eq!(candidates.len(), 1);
    assert!(candidates[0].cleanup_due);
    Ok(())
}
