use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

pub fn request(
    client: &mut Client,
    player_uuid: Uuid,
    target_server: &str,
    source: &str,
    location: Value,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_pending_teleports (player_uuid, target_server, source, location)
         values ($1, $2, $3, $4)
         on conflict (player_uuid) do update set
         target_server = excluded.target_server,
         source = excluded.source,
         location = excluded.location,
         created_at = now(),
         expires_at = now() + interval '60 seconds'",
        &[&player_uuid, &target_server, &source, &location],
    )?;
    Ok(())
}

pub fn take(
    client: &mut Client,
    player_uuid: Uuid,
    server_id: &str,
) -> Result<Option<Value>, StoreError> {
    let row = client.query_opt(
        "delete from player_pending_teleports
         where player_uuid = $1 and target_server = $2 and expires_at > now()
         returning location",
        &[&player_uuid, &server_id],
    )?;
    Ok(row.map(|row| row.get(0)))
}
