use postgres::{Client, GenericClient};
use serde_json::json;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresenceHeartbeat<'a> {
    pub instance_id: &'a str,
    pub player_count: Option<i32>,
    pub max_players: Option<i32>,
    pub ready: bool,
    pub implementation: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresenceRecord {
    pub instance_id: String,
    pub player_count: Option<i32>,
    pub max_players: Option<i32>,
    pub ready: bool,
    pub heartbeat_age_seconds: Option<i64>,
    pub empty_since_age_seconds: Option<i64>,
    pub suspend_reason: Option<String>,
}

pub fn upsert_heartbeat(
    client: &mut Client,
    heartbeat: PresenceHeartbeat<'_>,
) -> Result<(), StoreError> {
    upsert_heartbeat_in(client, heartbeat)
}

pub fn upsert_heartbeat_in(
    client: &mut impl GenericClient,
    heartbeat: PresenceHeartbeat<'_>,
) -> Result<(), StoreError> {
    let metadata = match heartbeat.implementation {
        Some(value) => json!({"implementation": value}),
        None => json!({}),
    };
    client.execute(
        "insert into instance_presence
         (instance_id, last_heartbeat_at, player_count, max_players, ready, last_nonempty_at, metadata)
         values ($1, now(), $2, $3, $4, case when $2 > 0 then now() else null end, $5)
         on conflict (instance_id) do update set
         last_heartbeat_at = now(),
         player_count = excluded.player_count,
         max_players = excluded.max_players,
         ready = excluded.ready,
         last_nonempty_at = case when excluded.player_count > 0 then now()
             else instance_presence.last_nonempty_at end,
         empty_since = case when excluded.player_count > 0 then null
             else instance_presence.empty_since end,
         metadata = instance_presence.metadata || excluded.metadata,
         updated_at = now()",
        &[
            &heartbeat.instance_id,
            &heartbeat.player_count,
            &heartbeat.max_players,
            &heartbeat.ready,
            &metadata,
        ],
    )?;
    Ok(())
}

pub fn get(client: &mut Client, instance_id: &str) -> Result<Option<PresenceRecord>, StoreError> {
    let row = client.query_opt(
        "select instance_id, player_count, max_players, ready,
         extract(epoch from now() - last_heartbeat_at)::bigint,
         case when empty_since is null then null else extract(epoch from now() - empty_since)::bigint end,
         suspend_reason
         from instance_presence where instance_id = $1",
        &[&instance_id],
    )?;
    Ok(row.map(record_from_row))
}

pub fn list(client: &mut Client) -> Result<Vec<PresenceRecord>, StoreError> {
    let rows = client.query(
        "select instance_id, player_count, max_players, ready,
         extract(epoch from now() - last_heartbeat_at)::bigint,
         case when empty_since is null then null else extract(epoch from now() - empty_since)::bigint end,
         suspend_reason
         from instance_presence order by instance_id",
        &[],
    )?;
    Ok(rows.into_iter().map(record_from_row).collect())
}

pub fn set_empty_since(client: &mut Client, instance_id: &str) -> Result<(), StoreError> {
    client.execute(
        "update instance_presence set empty_since = coalesce(empty_since, now()), updated_at = now()
         where instance_id = $1",
        &[&instance_id],
    )?;
    Ok(())
}

pub fn clear_empty_since(client: &mut Client, instance_id: &str) -> Result<(), StoreError> {
    client.execute(
        "update instance_presence set empty_since = null, updated_at = now()
         where instance_id = $1",
        &[&instance_id],
    )?;
    Ok(())
}

pub fn mark_autosuspended(
    client: &mut Client,
    instance_id: &str,
    reason: &str,
) -> Result<(), StoreError> {
    client.execute(
        "update instances set desired_state = 'suspended', updated_at = now() where id = $1",
        &[&instance_id],
    )?;
    client.execute(
        "update instance_presence set last_suspend_at = now(), suspend_reason = $2,
         updated_at = now() where instance_id = $1",
        &[&instance_id, &reason],
    )?;
    Ok(())
}

pub fn clear_autosuspended(client: &mut Client, instance_id: &str) -> Result<(), StoreError> {
    client.execute(
        "update instance_presence set empty_since = null, last_wake_at = now(),
         suspend_reason = null, updated_at = now() where instance_id = $1",
        &[&instance_id],
    )?;
    Ok(())
}

fn record_from_row(row: postgres::Row) -> PresenceRecord {
    PresenceRecord {
        instance_id: row.get(0),
        player_count: row.get(1),
        max_players: row.get(2),
        ready: row.get(3),
        heartbeat_age_seconds: row.get(4),
        empty_since_age_seconds: row.get(5),
        suspend_reason: row.get(6),
    }
}
