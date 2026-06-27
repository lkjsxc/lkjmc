use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceRecord {
    pub id: String,
    pub kind: String,
    pub desired_state: String,
    pub node_id: Option<Uuid>,
    pub observed_state: Option<String>,
    pub healthy: Option<bool>,
    pub pid: Option<i32>,
    pub message: Option<String>,
    pub uptime_seconds: Option<i64>,
}

pub fn insert(
    client: &mut Client,
    id: &str,
    node_id: Option<Uuid>,
    kind: &str,
    desired_state: &str,
    config: &Value,
) -> Result<(), StoreError> {
    client.execute(
        "insert into instances (id, node_id, kind, desired_state, config)
         values ($1, $2, $3, $4, $5)",
        &[&id, &node_id, &kind, &desired_state, &config],
    )?;
    Ok(())
}

pub fn list(client: &mut Client) -> Result<Vec<InstanceRecord>, StoreError> {
    let rows = client.query(
        "select i.id, i.kind, i.desired_state, i.node_id, o.observed_state,
         o.healthy, o.pid, o.message,
         case when o.started_at is null then null else extract(epoch from now() - o.started_at)::bigint end
         from instances i left join instance_observations o on o.instance_id = i.id
         order by i.id",
        &[],
    )?;
    Ok(rows.into_iter().map(record_from_row).collect())
}

pub fn get(client: &mut Client, id: &str) -> Result<Option<InstanceRecord>, StoreError> {
    let row = client.query_opt(
        "select i.id, i.kind, i.desired_state, i.node_id, o.observed_state,
         o.healthy, o.pid, o.message,
         case when o.started_at is null then null else extract(epoch from now() - o.started_at)::bigint end
         from instances i left join instance_observations o on o.instance_id = i.id
         where i.id = $1",
        &[&id],
    )?;
    Ok(row.map(record_from_row))
}

pub fn config(client: &mut Client, id: &str) -> Result<Option<Value>, StoreError> {
    let row = client.query_opt("select config from instances where id = $1", &[&id])?;
    Ok(row.map(|row| row.get(0)))
}

pub fn update_config(client: &mut Client, id: &str, config: &Value) -> Result<u64, StoreError> {
    Ok(client.execute(
        "update instances set config = $2, updated_at = now() where id = $1",
        &[&id, &config],
    )?)
}

pub fn set_jar_asset(client: &mut Client, id: &str, jar_asset_id: Uuid) -> Result<u64, StoreError> {
    Ok(client.execute(
        "update instances set jar_asset_id = $2, updated_at = now() where id = $1",
        &[&id, &jar_asset_id],
    )?)
}

pub fn update_desired_state(
    client: &mut Client,
    id: &str,
    desired_state: &str,
) -> Result<u64, StoreError> {
    Ok(client.execute(
        "update instances set desired_state = $2, updated_at = now() where id = $1",
        &[&id, &desired_state],
    )?)
}

pub fn delete(client: &mut Client, id: &str) -> Result<u64, StoreError> {
    Ok(client.execute("delete from instances where id = $1", &[&id])?)
}

pub fn reserve_port(
    client: &mut Client,
    instance_id: &str,
    port: i32,
    purpose: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into instance_ports (port, instance_id, purpose) values ($1, $2, $3)",
        &[&port, &instance_id, &purpose],
    )?;
    Ok(())
}

pub fn allocate_port(
    client: &mut Client,
    instance_id: &str,
    purpose: &str,
    range_start: i32,
    range_end: i32,
) -> Result<i32, StoreError> {
    let row = client.query_opt(
        "insert into instance_ports (port, instance_id, purpose)
         select candidate.port, $1, $2 from generate_series($3::integer, $4::integer) as candidate(port)
         where not exists (select 1 from instance_ports where port = candidate.port)
         order by candidate.port limit 1 returning port",
        &[&instance_id, &purpose, &range_start, &range_end],
    )?;
    row.map(|row| row.get(0))
        .ok_or_else(|| StoreError::invalid_state("no free port available"))
}

pub fn upsert_observation(
    client: &mut Client,
    instance_id: &str,
    observed_state: &str,
    pid: Option<i32>,
    healthy: bool,
    message: Option<&str>,
) -> Result<(), StoreError> {
    client.execute(
        "insert into instance_observations
         (instance_id, observed_state, pid, healthy, started_at, message)
         values ($1, $2, $3, $4, case when $4 then now() else null end, $5)
         on conflict (instance_id) do update set
         observed_state = excluded.observed_state,
         pid = excluded.pid,
         healthy = excluded.healthy,
         started_at = case when excluded.healthy then coalesce(instance_observations.started_at, now()) else instance_observations.started_at end,
         message = excluded.message,
         updated_at = now()",
        &[&instance_id, &observed_state, &pid, &healthy, &message],
    )?;
    Ok(())
}

fn record_from_row(row: postgres::Row) -> InstanceRecord {
    InstanceRecord {
        id: row.get(0),
        kind: row.get(1),
        desired_state: row.get(2),
        node_id: row.get(3),
        observed_state: row.get(4),
        healthy: row.get(5),
        pid: row.get(6),
        message: row.get(7),
        uptime_seconds: row.get(8),
    }
}
