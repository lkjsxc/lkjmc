use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

pub fn set_language(
    client: &mut Client,
    player_uuid: Uuid,
    language: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_settings (player_uuid, language)
         values ($1, $2)
         on conflict (player_uuid) do update set
         language = excluded.language,
         updated_at = now()",
        &[&player_uuid, &language],
    )?;
    Ok(())
}

pub fn set_hud(client: &mut Client, player_uuid: Uuid, enabled: bool) -> Result<(), StoreError> {
    client.execute(
        "insert into player_settings (player_uuid, language, hud_enabled)
         values ($1, 'en', $2)
         on conflict (player_uuid) do update set
         hud_enabled = excluded.hud_enabled,
         updated_at = now()",
        &[&player_uuid, &enabled],
    )?;
    Ok(())
}

pub fn language(client: &mut Client, player_uuid: Uuid) -> Result<Option<String>, StoreError> {
    let row = client.query_opt(
        "select language from player_settings where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| row.get(0)))
}
