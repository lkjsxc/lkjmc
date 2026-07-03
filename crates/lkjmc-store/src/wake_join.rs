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
    pub correlation_id: &'a str,
    pub metadata: Value,
}

pub fn create_or_live(
    client: &mut Client,
    new: NewWakeJoin<'_>,
) -> Result<WakeJoinRecord, StoreError> {
    crate::player::ensure_identity(client, new.player_uuid, Some(new.player_name))?;
    if let Some(record) = live_for(client, new.player_uuid, new.target_instance_id)? {
        return Ok(record);
    }
    let row = client.query_one(
        "insert into wake_join_queue
         (id, player_uuid, player_name, target_instance_id, requested_by_kind,
          requested_by_name, state, expires_at, correlation_id, metadata)
         values ($1, $2, $3, $4, $5, $6, 'queued',
          now() + ($7::integer * interval '1 second'), $8, $9)
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
            &new.correlation_id,
            &new.metadata,
        ],
    )?;
    Ok(record_from_row(row))
}

pub fn mark_starting(client: &mut Client, id: Uuid) -> Result<(), StoreError> {
    transition(client, id, "starting", &["queued"])
}

pub fn mark_ready(client: &mut Client, id: Uuid, target_server: &str) -> Result<(), StoreError> {
    client.execute(
        "update wake_join_queue set state = 'ready', target_server = $2,
         updated_at = now() where id = $1 and state in ('queued', 'starting', 'ready')",
        &[&id, &target_server],
    )?;
    Ok(())
}

pub fn mark_failed(client: &mut Client, id: Uuid, reason: &str) -> Result<(), StoreError> {
    client.execute(
        "update wake_join_queue set state = 'failed', failure_reason = $2,
         cleanup_after = now() + interval '10 minutes', updated_at = now()
         where id = $1 and state not in ('transferred', 'cancelled', 'expired')",
        &[&id, &reason],
    )?;
    Ok(())
}

pub fn cancel(
    client: &mut Client,
    id: Uuid,
    player_uuid: Uuid,
) -> Result<Option<WakeJoinRecord>, StoreError> {
    client.execute(
        "update wake_join_queue set state = 'cancelled', cancelled_at = now(),
         cleanup_after = now() + interval '10 minutes', updated_at = now()
         where id = $1 and player_uuid = $2 and state in ('queued', 'starting', 'ready')",
        &[&id, &player_uuid],
    )?;
    get(client, id)
}

pub fn consume_ready(
    client: &mut Client,
    id: Uuid,
    target_server: &str,
) -> Result<Option<WakeJoinRecord>, StoreError> {
    let row = client.query_opt(
        "update wake_join_queue set state = 'transferred', consumed_at = now(),
         target_server = $2, cleanup_after = now() + interval '10 minutes', updated_at = now()
         where id = $1 and state = 'ready'
         returning id, player_uuid, target_instance_id, state, target_server, failure_reason",
        &[&id, &target_server],
    )?;
    Ok(row.map(record_from_row))
}

pub fn expire_due(client: &mut Client) -> Result<u64, StoreError> {
    Ok(client.execute(
        "update wake_join_queue set state = 'expired', cleanup_after = now() + interval '10 minutes',
         updated_at = now() where expires_at < now() and state in ('queued', 'starting', 'ready')",
        &[],
    )?)
}

pub fn get(client: &mut Client, id: Uuid) -> Result<Option<WakeJoinRecord>, StoreError> {
    let row = client.query_opt(
        "select id, player_uuid, target_instance_id, state, target_server,
         failure_reason from wake_join_queue where id = $1",
        &[&id],
    )?;
    Ok(row.map(record_from_row))
}

pub fn live_for(
    client: &mut Client,
    player: Uuid,
    target: &str,
) -> Result<Option<WakeJoinRecord>, StoreError> {
    let row = client.query_opt(
        "select id, player_uuid, target_instance_id, state, target_server, failure_reason
         from wake_join_queue where player_uuid = $1 and target_instance_id = $2
         and state in ('queued', 'starting', 'ready') and expires_at > now()
         order by created_at desc limit 1",
        &[&player, &target],
    )?;
    Ok(row.map(record_from_row))
}

fn transition(client: &mut Client, id: Uuid, state: &str, from: &[&str]) -> Result<(), StoreError> {
    client.execute(
        "update wake_join_queue set state = $2, updated_at = now() where id = $1 and state = any($3)",
        &[&id, &state, &from],
    )?;
    Ok(())
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
