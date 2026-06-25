use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

pub fn insert(
    client: &mut Client,
    id: Uuid,
    player_uuid: Uuid,
    current_server: &str,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into player_sessions (id, player_uuid, current_server, metadata)
         values ($1, $2, $3, $4)",
        &[&id, &player_uuid, &current_server, &metadata],
    )?;
    Ok(())
}

pub fn leave(
    client: &mut Client,
    player_uuid: Uuid,
    current_server: &str,
) -> Result<(), StoreError> {
    client.execute(
        "update player_sessions set left_at = now()
         where player_uuid = $1 and current_server = $2 and left_at is null",
        &[&player_uuid, &current_server],
    )?;
    Ok(())
}

pub fn active_count_for_server(
    client: &mut Client,
    current_server: &str,
) -> Result<i64, StoreError> {
    let row = client.query_one(
        "select count(*)::bigint from player_sessions
         where current_server = $1 and left_at is null",
        &[&current_server],
    )?;
    Ok(row.get(0))
}
