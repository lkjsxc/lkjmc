use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

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

pub fn snapshot_count(client: &mut Client, player_uuid: Uuid) -> Result<i64, StoreError> {
    let row = client.query_one(
        "select count(*)::bigint from player_profile_snapshots where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.get(0))
}
