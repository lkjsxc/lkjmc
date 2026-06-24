use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRecord {
    pub id: Uuid,
    pub name: String,
    pub hostname: String,
    pub kind: String,
}

pub fn insert(
    client: &mut Client,
    id: Uuid,
    name: &str,
    hostname: &str,
    kind: &str,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into nodes (id, name, hostname, kind, metadata)
         values ($1, $2, $3, $4, $5)",
        &[&id, &name, &hostname, &kind, &metadata],
    )?;
    Ok(())
}

pub fn get(client: &mut Client, id: Uuid) -> Result<Option<NodeRecord>, StoreError> {
    let row = client.query_opt(
        "select id, name, hostname, kind from nodes where id = $1",
        &[&id],
    )?;
    Ok(row.map(|row| NodeRecord {
        id: row.get(0),
        name: row.get(1),
        hostname: row.get(2),
        kind: row.get(3),
    }))
}
