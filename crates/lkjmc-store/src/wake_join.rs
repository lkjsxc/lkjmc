use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeJoinRecord {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub target_instance_id: String,
    pub state: String,
    pub target_server: Option<String>,
    pub failure_reason: Option<String>,
}

pub struct NewWakeJoin<'a> {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub player_name: &'a str,
    pub target_instance_id: &'a str,
    pub requested_by_kind: &'a str,
    pub requested_by_name: &'a str,
    pub expires_in_seconds: i32,
    pub metadata: Value,
}

pub fn create(client: &mut Client, new: NewWakeJoin<'_>) -> Result<WakeJoinRecord, StoreError> {
    let row = client.query_one(
        "insert into wake_join_queue
         (id, player_uuid, player_name, target_instance_id, requested_by_kind,
          requested_by_name, state, expires_at, metadata)
         values ($1, $2, $3, $4, $5, $6, 'queued',
          now() + ($7::text || ' seconds')::interval, $8)
         returning id, player_uuid, target_instance_id, state, target_server,
          failure_reason",
        &[
            &new.id,
            &new.player_uuid,
            &new.player_name,
            &new.target_instance_id,
            &new.requested_by_kind,
            &new.requested_by_name,
            &new.expires_in_seconds,
            &new.metadata,
        ],
    )?;
    Ok(record_from_row(row))
}

pub fn mark_waking(client: &mut Client, id: Uuid) -> Result<(), StoreError> {
    client.execute(
        "update wake_join_queue set state = 'waking', updated_at = now()
         where id = $1 and state = 'queued'",
        &[&id],
    )?;
    Ok(())
}

pub fn mark_ready(client: &mut Client, id: Uuid, target_server: &str) -> Result<(), StoreError> {
    client.execute(
        "update wake_join_queue set state = 'ready', target_server = $2,
         updated_at = now() where id = $1",
        &[&id, &target_server],
    )?;
    Ok(())
}

pub fn mark_failed(client: &mut Client, id: Uuid, reason: &str) -> Result<(), StoreError> {
    client.execute(
        "update wake_join_queue set state = 'failed', failure_reason = $2,
         updated_at = now() where id = $1",
        &[&id, &reason],
    )?;
    Ok(())
}

pub fn get(client: &mut Client, id: Uuid) -> Result<Option<WakeJoinRecord>, StoreError> {
    let row = client.query_opt(
        "select id, player_uuid, target_instance_id, state, target_server,
         failure_reason from wake_join_queue where id = $1",
        &[&id],
    )?;
    Ok(row.map(record_from_row))
}

fn record_from_row(row: postgres::Row) -> WakeJoinRecord {
    WakeJoinRecord {
        id: row.get(0),
        player_uuid: row.get(1),
        target_instance_id: row.get(2),
        state: row.get(3),
        target_server: row.get(4),
        failure_reason: row.get(5),
    }
}
