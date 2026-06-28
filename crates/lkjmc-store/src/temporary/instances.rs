use postgres::{GenericClient, Row};
use serde_json::Value;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporaryInstanceRecord {
    pub instance_id: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub lifecycle_state: String,
    pub cleanup_policy: String,
    pub world_path: String,
    pub server_port: i32,
}

pub struct NewTemporaryInstance<'a> {
    pub instance_id: &'a str,
    pub owner_kind: &'a str,
    pub owner_id: &'a str,
    pub visibility: &'a str,
    pub world_path: &'a str,
    pub server_port: i32,
    pub max_lifetime_seconds: i32,
    pub retention_seconds: i32,
    pub cleanup_policy: &'a str,
    pub lifecycle_state: &'a str,
    pub start_deadline_seconds: i32,
    pub metadata: Value,
}

pub fn insert_instance(
    client: &mut impl GenericClient,
    new: NewTemporaryInstance<'_>,
) -> Result<TemporaryInstanceRecord, StoreError> {
    let row = client.query_one(
        "insert into temporary_instances
         (instance_id, owner_kind, owner_id, visibility, world_path, server_port,
          max_lifetime_seconds, retention_seconds, cleanup_policy, lifecycle_state,
          start_deadline_at, stop_deadline_at, expires_at, retain_until, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
          now() + ($11::text || ' seconds')::interval,
          now() + ($7::text || ' seconds')::interval,
          now() + ($7::text || ' seconds')::interval,
          now() + (($7 + $8)::text || ' seconds')::interval, $12)
         returning instance_id, owner_kind, owner_id, lifecycle_state,
          cleanup_policy, world_path, server_port",
        &[
            &new.instance_id,
            &new.owner_kind,
            &new.owner_id,
            &new.visibility,
            &new.world_path,
            &new.server_port,
            &new.max_lifetime_seconds,
            &new.retention_seconds,
            &new.cleanup_policy,
            &new.lifecycle_state,
            &new.start_deadline_seconds,
            &new.metadata,
        ],
    )?;
    Ok(instance_from_row(&row))
}

pub fn get_instance(
    client: &mut impl GenericClient,
    instance_id: &str,
) -> Result<Option<TemporaryInstanceRecord>, StoreError> {
    let row = client.query_opt(
        "select instance_id, owner_kind, owner_id, lifecycle_state,
         cleanup_policy, world_path, server_port from temporary_instances
         where instance_id = $1",
        &[&instance_id],
    )?;
    Ok(row.map(|row| instance_from_row(&row)))
}

pub fn update_instance_state(
    client: &mut impl GenericClient,
    instance_id: &str,
    lifecycle_state: &str,
    last_error: Option<&str>,
) -> Result<(), StoreError> {
    client.execute(
        "update temporary_instances set lifecycle_state = $2, last_error = $3,
         updated_at = now() where instance_id = $1",
        &[&instance_id, &lifecycle_state, &last_error],
    )?;
    Ok(())
}

fn instance_from_row(row: &Row) -> TemporaryInstanceRecord {
    TemporaryInstanceRecord {
        instance_id: row.get(0),
        owner_kind: row.get(1),
        owner_id: row.get(2),
        lifecycle_state: row.get(3),
        cleanup_policy: row.get(4),
        world_path: row.get(5),
        server_port: row.get(6),
    }
}
