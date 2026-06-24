use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq)]
pub struct HomeRecord {
    pub name: String,
    pub server_id: String,
    pub location: Value,
}

pub fn upsert(
    client: &mut Client,
    id: Uuid,
    player_uuid: Uuid,
    name: &str,
    server_id: &str,
    location: Value,
) -> Result<(), StoreError> {
    client.execute(
        "insert into homes (id, player_uuid, name, server_id, location)
         values ($1, $2, $3, $4, $5)
         on conflict (player_uuid, name) do update set
         server_id = excluded.server_id,
         location = excluded.location",
        &[&id, &player_uuid, &name, &server_id, &location],
    )?;
    Ok(())
}

pub fn get(
    client: &mut Client,
    player_uuid: Uuid,
    name: &str,
) -> Result<Option<HomeRecord>, StoreError> {
    let row = client.query_opt(
        "select name, server_id, location from homes where player_uuid = $1 and name = $2",
        &[&player_uuid, &name],
    )?;
    Ok(row.map(|row| HomeRecord {
        name: row.get(0),
        server_id: row.get(1),
        location: row.get(2),
    }))
}
