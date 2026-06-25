use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerWarning {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub player_name: String,
    pub actor_name: String,
    pub reason: String,
}

pub fn create(
    client: &mut Client,
    id: Uuid,
    player_uuid: Uuid,
    player_name: &str,
    actor_name: &str,
    reason: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_warnings (id, player_uuid, player_name, actor_name, reason)
         values ($1, $2, $3, $4, $5)",
        &[&id, &player_uuid, &player_name, &actor_name, &reason],
    )?;
    Ok(())
}

pub fn list(
    client: &mut Client,
    player_uuid: Uuid,
    limit: i64,
) -> Result<Vec<PlayerWarning>, StoreError> {
    let rows = client.query(
        "select id, player_uuid, player_name, actor_name, reason
         from player_warnings where player_uuid = $1 order by created_at desc limit $2",
        &[&player_uuid, &limit],
    )?;
    Ok(rows.into_iter().map(warning_from_row).collect())
}

fn warning_from_row(row: postgres::Row) -> PlayerWarning {
    PlayerWarning {
        id: row.get(0),
        player_uuid: row.get(1),
        player_name: row.get(2),
        actor_name: row.get(3),
        reason: row.get(4),
    }
}
