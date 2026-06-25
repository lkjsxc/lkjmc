use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Punishment {
    pub id: Uuid,
    pub player_uuid: Uuid,
    pub player_name: String,
    pub reason: String,
    pub actor_name: String,
}

pub fn ban(
    client: &mut Client,
    id: Uuid,
    player_uuid: Uuid,
    player_name: &str,
    actor_name: &str,
    reason: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_punishments (id, player_uuid, player_name, kind, actor_name, reason)
         values ($1, $2, $3, 'ban', $4, $5)",
        &[&id, &player_uuid, &player_name, &actor_name, &reason],
    )?;
    Ok(())
}

pub fn revoke_ban(client: &mut Client, player_name: &str) -> Result<u64, StoreError> {
    Ok(client.execute(
        "update player_punishments set revoked_at = now()
         where lower(player_name) = lower($1) and kind = 'ban' and revoked_at is null",
        &[&player_name],
    )?)
}

pub fn active_ban(
    client: &mut Client,
    player_uuid: Uuid,
) -> Result<Option<Punishment>, StoreError> {
    let row = client.query_opt(
        "select id, player_uuid, player_name, reason, actor_name from player_punishments
         where player_uuid = $1 and kind = 'ban' and revoked_at is null
         and (expires_at is null or expires_at > now()) order by created_at desc limit 1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| Punishment {
        id: row.get(0),
        player_uuid: row.get(1),
        player_name: row.get(2),
        reason: row.get(3),
        actor_name: row.get(4),
    }))
}
