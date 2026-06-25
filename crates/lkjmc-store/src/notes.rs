use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerNote {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub player_name: String,
    pub actor_name: String,
    pub body: String,
}

pub fn create(
    client: &mut Client,
    id: Uuid,
    player_uuid: Uuid,
    player_name: &str,
    actor_name: &str,
    body: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_notes (id, player_uuid, player_name, actor_name, body)
         values ($1, $2, $3, $4, $5)",
        &[&id, &player_uuid, &player_name, &actor_name, &body],
    )?;
    Ok(())
}

pub fn list(
    client: &mut Client,
    player_uuid: Uuid,
    limit: i64,
) -> Result<Vec<PlayerNote>, StoreError> {
    let rows = client.query(
        "select id, player_uuid, player_name, actor_name, body
         from player_notes where player_uuid = $1 order by created_at desc limit $2",
        &[&player_uuid, &limit],
    )?;
    Ok(rows.into_iter().map(note_from_row).collect())
}

fn note_from_row(row: postgres::Row) -> PlayerNote {
    PlayerNote {
        id: row.get(0),
        player_uuid: row.get(1),
        player_name: row.get(2),
        actor_name: row.get(3),
        body: row.get(4),
    }
}
