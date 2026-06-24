use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyRecord {
    pub id: Uuid,
    pub name: Option<String>,
    pub role: String,
}

pub fn create(
    client: &mut Client,
    party_id: Uuid,
    owner_uuid: Uuid,
    name: &str,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into parties (id, owner_uuid, name, metadata) values ($1, $2, $3, $4)",
        &[&party_id, &owner_uuid, &name, &metadata],
    )?;
    client.execute(
        "insert into party_members (party_id, player_uuid, role) values ($1, $2, 'owner')",
        &[&party_id, &owner_uuid],
    )?;
    Ok(())
}

pub fn current(client: &mut Client, player_uuid: Uuid) -> Result<Option<PartyRecord>, StoreError> {
    let row = client.query_opt(
        "select p.id, p.name, m.role from parties p
         join party_members m on m.party_id = p.id
         where m.player_uuid = $1 order by m.joined_at desc limit 1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| PartyRecord {
        id: row.get(0),
        name: row.get(1),
        role: row.get(2),
    }))
}

pub fn leave(client: &mut Client, player_uuid: Uuid) -> Result<u64, StoreError> {
    Ok(client.execute(
        "delete from party_members where player_uuid = $1",
        &[&player_uuid],
    )?)
}

pub fn delete_empty(client: &mut Client) -> Result<u64, StoreError> {
    Ok(client.execute(
        "delete from parties p where not exists
         (select 1 from party_members m where m.party_id = p.id)",
        &[],
    )?)
}
