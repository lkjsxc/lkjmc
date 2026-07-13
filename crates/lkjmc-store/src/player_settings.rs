use postgres::{Client, GenericClient};
use uuid::Uuid;

use crate::error::StoreError;

pub struct Settings {
    pub hud_enabled: bool,
    pub language: String,
    pub menu_enabled: bool,
}

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

pub fn set_language_for_identity<C: GenericClient>(
    client: &mut C,
    player_uuid: Uuid,
    name: &str,
    language: &str,
) -> Result<(), StoreError> {
    client.execute(
        "with identity as (
           insert into player_identities (player_uuid, current_name, metadata)
           values ($1, $2, '{}'::jsonb)
           on conflict (player_uuid) do update set current_name = excluded.current_name,
           last_seen_at = now()
           returning player_uuid
         ) insert into player_settings (player_uuid, language)
         select player_uuid, $3 from identity
         on conflict (player_uuid) do update set language = excluded.language,
         updated_at = now()",
        &[&player_uuid, &name, &language],
    )?;
    Ok(())
}

pub fn set_hud_for_identity<C: GenericClient>(
    client: &mut C,
    player_uuid: Uuid,
    name: &str,
    enabled: bool,
) -> Result<(), StoreError> {
    client.execute(
        "with identity as (
           insert into player_identities (player_uuid, current_name, metadata)
           values ($1, $2, '{}'::jsonb)
           on conflict (player_uuid) do update set current_name = excluded.current_name,
           last_seen_at = now()
           returning player_uuid
         ) insert into player_settings (player_uuid, language, hud_enabled)
         select player_uuid, 'en', $3 from identity
         on conflict (player_uuid) do update set hud_enabled = excluded.hud_enabled,
         updated_at = now()",
        &[&player_uuid, &name, &enabled],
    )?;
    Ok(())
}

pub fn current(client: &mut Client, player_uuid: Uuid) -> Result<Option<Settings>, StoreError> {
    let row = client.query_opt(
        "select language, hud_enabled, menu_enabled from player_settings where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| Settings {
        language: row.get(0),
        hud_enabled: row.get(1),
        menu_enabled: row.get(2),
    }))
}

pub fn set_menu_enabled(
    client: &mut Client,
    player_uuid: Uuid,
    enabled: bool,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_settings (player_uuid, language, menu_enabled)
         values ($1, 'en', $2)
         on conflict (player_uuid) do update set
         menu_enabled = excluded.menu_enabled,
         updated_at = now()",
        &[&player_uuid, &enabled],
    )?;
    Ok(())
}

pub fn hud_enabled(client: &mut Client, player_uuid: Uuid) -> Result<Option<bool>, StoreError> {
    let row = client.query_opt(
        "select hud_enabled from player_settings where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| row.get(0)))
}

pub fn language(client: &mut Client, player_uuid: Uuid) -> Result<Option<String>, StoreError> {
    let row = client.query_opt(
        "select language from player_settings where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| row.get(0)))
}

pub fn menu_enabled(client: &mut Client, player_uuid: Uuid) -> Result<Option<bool>, StoreError> {
    let row = client.query_opt(
        "select menu_enabled from player_settings where player_uuid = $1",
        &[&player_uuid],
    )?;
    Ok(row.map(|row| row.get(0)))
}

pub fn toggle_hud(client: &mut Client, player_uuid: Uuid) -> Result<bool, StoreError> {
    toggle_bool(client, player_uuid, "hud_enabled", true)
}

pub fn toggle_menu_enabled(client: &mut Client, player_uuid: Uuid) -> Result<bool, StoreError> {
    toggle_bool(client, player_uuid, "menu_enabled", false)
}

fn toggle_bool(
    client: &mut Client,
    player_uuid: Uuid,
    column: &str,
    inserted_value: bool,
) -> Result<bool, StoreError> {
    let query = format!(
        "insert into player_settings (player_uuid, language, {column})
         values ($1, 'en', $2)
         on conflict (player_uuid) do update set
         {column} = not player_settings.{column}, updated_at = now()
         returning {column}"
    );
    let row = client.query_one(&query, &[&player_uuid, &inserted_value])?;
    Ok(row.get(0))
}
