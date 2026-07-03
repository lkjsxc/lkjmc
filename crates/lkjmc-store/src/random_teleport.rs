use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq)]
pub struct RandomTeleportRecord {
    pub correlation_id: Uuid,
    pub server_id: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub cost_points: i64,
    pub state: String,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReserveOutcome {
    Reserved,
    Existing(String),
    InsufficientPoints,
}

pub struct ReserveInput<'a> {
    pub id: Uuid,
    pub correlation_id: Uuid,
    pub player_uuid: Uuid,
    pub server_id: &'a str,
    pub world: &'a str,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub cost_points: i64,
}

pub fn cooldown_remaining(
    client: &mut Client,
    player_uuid: Uuid,
    server_id: &str,
    cooldown_seconds: i64,
) -> Result<i64, StoreError> {
    let row = client.query_opt(
        "select greatest(0, $3::bigint - extract(epoch from (now() - created_at))::bigint)
         from random_teleports
         where player_uuid = $1 and server_id = $2 and state in ('reserved', 'completed')
         order by created_at desc limit 1",
        &[&player_uuid, &server_id, &cooldown_seconds],
    )?;
    Ok(row.map(|row| row.get(0)).unwrap_or(0))
}

pub fn reserve(client: &mut Client, input: ReserveInput<'_>) -> Result<ReserveOutcome, StoreError> {
    let mut tx = client.transaction()?;
    if let Some(row) = tx.query_opt(
        "select state from random_teleports where correlation_id = $1",
        &[&input.correlation_id],
    )? {
        let state: String = row.get(0);
        tx.commit()?;
        return Ok(ReserveOutcome::Existing(state));
    }
    let spent = crate::points::spend_with_correlation(
        &mut tx,
        input.player_uuid,
        input.cost_points,
        "random-teleport",
        Some(input.correlation_id),
    )?;
    if spent.is_none() {
        tx.commit()?;
        return Ok(ReserveOutcome::InsufficientPoints);
    }
    let metadata = serde_json::Value::Object(Default::default());
    tx.execute(
        "insert into random_teleports
         (id, correlation_id, player_uuid, server_id, world, x, y, z, cost_points, state, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'reserved', $10)",
        &[
            &input.id,
            &input.correlation_id,
            &input.player_uuid,
            &input.server_id,
            &input.world,
            &input.x,
            &input.y,
            &input.z,
            &input.cost_points,
            &metadata,
        ],
    )?;
    tx.commit()?;
    Ok(ReserveOutcome::Reserved)
}

pub fn complete(
    client: &mut Client,
    player_uuid: Uuid,
    correlation_id: Uuid,
) -> Result<bool, StoreError> {
    let updated = client.execute(
        "update random_teleports set state = 'completed', completed_at = now()
         where player_uuid = $1 and correlation_id = $2 and state = 'reserved'",
        &[&player_uuid, &correlation_id],
    )?;
    Ok(updated > 0)
}

pub fn refund(
    client: &mut Client,
    player_uuid: Uuid,
    correlation_id: Uuid,
    reason: &str,
) -> Result<bool, StoreError> {
    let mut tx = client.transaction()?;
    let Some(row) = tx.query_opt(
        "select cost_points, state from random_teleports
         where player_uuid = $1 and correlation_id = $2 for update",
        &[&player_uuid, &correlation_id],
    )?
    else {
        tx.commit()?;
        return Ok(false);
    };
    let cost: i64 = row.get(0);
    let state: String = row.get(1);
    if state == "refunded" || state == "completed" {
        tx.commit()?;
        return Ok(false);
    }
    crate::points::grant_with_correlation(
        &mut tx,
        player_uuid,
        cost,
        "random-teleport-refund",
        Some(refund_correlation(correlation_id)),
    )?;
    tx.execute(
        "update random_teleports
         set state = 'refunded', failure_reason = $3, refunded_at = now()
         where player_uuid = $1 and correlation_id = $2",
        &[&player_uuid, &correlation_id, &reason],
    )?;
    tx.commit()?;
    Ok(true)
}

pub fn history(
    client: &mut Client,
    player_uuid: Uuid,
) -> Result<Vec<RandomTeleportRecord>, StoreError> {
    let rows = client.query(
        "select correlation_id, server_id, world, x, y, z, cost_points, state, failure_reason
         from random_teleports where player_uuid = $1 order by created_at desc limit 10",
        &[&player_uuid],
    )?;
    Ok(rows.into_iter().map(record).collect())
}

fn refund_correlation(correlation_id: Uuid) -> Uuid {
    Uuid::new_v5(&correlation_id, b"random-teleport-refund")
}

fn record(row: postgres::Row) -> RandomTeleportRecord {
    RandomTeleportRecord {
        correlation_id: row.get(0),
        server_id: row.get(1),
        world: row.get(2),
        x: row.get(3),
        y: row.get(4),
        z: row.get(5),
        cost_points: row.get(6),
        state: row.get(7),
        failure_reason: row.get(8),
    }
}
