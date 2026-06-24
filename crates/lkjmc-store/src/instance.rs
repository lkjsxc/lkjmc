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

pub fn get(client: &mut Client, id: &str) -> Result<Option<InstanceRecord>, StoreError> {
    let row = client.query_opt(
        "select id, kind, desired_state, node_id from instances where id = $1",
        &[&id],
    )?;
    Ok(row.map(|row| InstanceRecord {
        id: row.get(0),
        kind: row.get(1),
        desired_state: row.get(2),
        node_id: row.get(3),
    }))
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
         (instance_id, observed_state, pid, healthy, message)
         values ($1, $2, $3, $4, $5)
         on conflict (instance_id) do update set
         observed_state = excluded.observed_state,
         pid = excluded.pid,
         healthy = excluded.healthy,
         message = excluded.message,
         updated_at = now()",
        &[&instance_id, &observed_state, &pid, &healthy, &message],
    )?;
    Ok(())
}
