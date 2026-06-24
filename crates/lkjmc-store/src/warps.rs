use postgres::Client;
use serde_json::Value;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq)]
pub struct WarpRecord {
    pub name: String,
    pub server_id: String,
    pub location: Value,
}

pub fn upsert(
    client: &mut Client,
    name: &str,
    server_id: &str,
    location: Value,
) -> Result<(), StoreError> {
    client.execute(
        "insert into warps (name, server_id, location)
         values ($1, $2, $3)
         on conflict (name) do update set
         server_id = excluded.server_id,
         location = excluded.location",
        &[&name, &server_id, &location],
    )?;
    Ok(())
}

pub fn get(client: &mut Client, name: &str) -> Result<Option<WarpRecord>, StoreError> {
    let row = client.query_opt(
        "select name, server_id, location from warps where name = $1",
        &[&name],
    )?;
    Ok(row.map(|row| WarpRecord {
        name: row.get(0),
        server_id: row.get(1),
        location: row.get(2),
    }))
}
