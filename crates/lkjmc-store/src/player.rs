use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRecord {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub scope: String,
    pub revision: i64,
    pub payload: Vec<u8>,
    pub sha256: String,
    pub payload_format: String,
}

pub fn insert_identity(
    client: &mut Client,
    player_uuid: Uuid,
    name: &str,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into player_identities (player_uuid, current_name, metadata)
         values ($1, $2, $3)
         on conflict (player_uuid) do update set
         current_name = excluded.current_name,
         last_seen_at = now()",
        &[&player_uuid, &name, &metadata],
    )?;
    Ok(())
}

pub fn get_identity_name(
    client: &mut Client,
    player_uuid: Uuid,
) -> Result<Option<String>, StoreError> {
    let row = client.query_opt(
        "select current_name from player_identities where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| row.get(0)))
}

pub fn acquire_lease(
    client: &mut Client,
    player_uuid: Uuid,
    scope: &str,
    holder: &str,
) -> Result<i64, StoreError> {
    let row = client.query_one(
        "insert into player_profile_leases
         (player_uuid, scope, holder, revision, expires_at)
         values ($1, $2, $3, 0, now() + interval '30 seconds')
         on conflict (player_uuid, scope) do update set
         holder = excluded.holder,
         expires_at = excluded.expires_at,
         updated_at = now()
         where player_profile_leases.expires_at < now()
            or player_profile_leases.holder = excluded.holder
         returning revision",
        &[&player_uuid, &scope, &holder],
    )?;
    Ok(row.get(0))
}

pub fn upsert_lease(
    client: &mut Client,
    player_uuid: Uuid,
    scope: &str,
    holder: &str,
    revision: i64,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_profile_leases
         (player_uuid, scope, holder, revision, expires_at)
         values ($1, $2, $3, $4, now() + interval '30 seconds')
         on conflict (player_uuid, scope) do update set
         holder = excluded.holder,
         revision = excluded.revision,
         expires_at = excluded.expires_at,
         updated_at = now()",
        &[&player_uuid, &scope, &holder, &revision],
    )?;
    Ok(())
}

pub fn insert_snapshot(
    client: &mut Client,
    id: Uuid,
    player_uuid: Uuid,
    scope: &str,
    revision: i64,
    payload: &[u8],
    sha256: &str,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into player_profile_snapshots
         (id, player_uuid, scope, revision, payload_format, payload, sha256, source_instance, metadata)
         values ($1, $2, $3, $4, 'test-bytes', $5, $6, 'test-source', $7)",
        &[&id, &player_uuid, &scope, &revision, &payload, &sha256, &metadata],
    )?;
    Ok(())
}

pub fn insert_snapshot_with_metadata(
    client: &mut Client,
    snapshot: NewSnapshot<'_>,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_profile_snapshots
         (id, player_uuid, scope, revision, payload_format, payload, sha256, source_instance, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        &[
            &snapshot.id,
            &snapshot.player_uuid,
            &snapshot.scope,
            &snapshot.revision,
            &snapshot.payload_format,
            &snapshot.payload,
            &snapshot.sha256,
            &snapshot.source_instance,
            &snapshot.metadata,
        ],
    )?;
    Ok(())
}

pub fn latest_snapshot(
    client: &mut Client,
    player_uuid: Uuid,
    scope: &str,
) -> Result<Option<SnapshotRecord>, StoreError> {
    let row = client.query_opt(
        "select id, player_uuid, scope, revision, payload, sha256, payload_format
         from player_profile_snapshots
         where player_uuid = $1 and scope = $2
         order by revision desc limit 1",
        &[&player_uuid, &scope],
    )?;
    Ok(row.map(|row| SnapshotRecord {
        id: row.get(0),
        player_uuid: row.get(1),
        scope: row.get(2),
        revision: row.get(3),
        payload: row.get(4),
        sha256: row.get(5),
        payload_format: row.get(6),
    }))
}

pub fn snapshot_by_id(
    client: &mut Client,
    snapshot_id: Uuid,
    player_uuid: Uuid,
    scope: &str,
) -> Result<Option<SnapshotRecord>, StoreError> {
    let row = client.query_opt(
        "select id, player_uuid, scope, revision, payload, sha256, payload_format
         from player_profile_snapshots
         where id = $1 and player_uuid = $2 and scope = $3",
        &[&snapshot_id, &player_uuid, &scope],
    )?;
    Ok(row.map(|row| SnapshotRecord {
        id: row.get(0),
        player_uuid: row.get(1),
        scope: row.get(2),
        revision: row.get(3),
        payload: row.get(4),
        sha256: row.get(5),
        payload_format: row.get(6),
    }))
}

pub struct NewSnapshot<'a> {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub scope: &'a str,
    pub revision: i64,
    pub payload_format: &'a str,
    pub payload: &'a [u8],
    pub sha256: &'a str,
    pub source_instance: &'a str,
    pub metadata: Value,
}

pub fn snapshot_count(client: &mut Client, player_uuid: Uuid) -> Result<i64, StoreError> {
    let row = client.query_one(
        "select count(*)::bigint from player_profile_snapshots where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.get(0))
}
