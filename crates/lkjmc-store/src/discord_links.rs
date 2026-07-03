use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordLinkRecord {
    pub discord_user_id: String,
    pub minecraft_uuid: Uuid,
    pub verification_state: String,
    pub verified: bool,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkCodeRecord {
    pub player_uuid: Uuid,
    pub player_name: String,
}

pub fn begin(
    client: &mut Client,
    player_uuid: Uuid,
    player_name: &str,
    code_hash: &str,
    minutes: i32,
) -> Result<(), StoreError> {
    let mut tx = client.transaction()?;
    tx.execute(
        "update link_codes set consumed_at = now()
         where player_uuid = $1 and consumed_at is null",
        &[&player_uuid],
    )?;
    tx.execute(
        "insert into link_codes (id, player_uuid, player_name, code_hash, expires_at)
         values ($1, $2, $3, $4, now() + ($5::int * interval '1 minute'))",
        &[
            &Uuid::new_v4(),
            &player_uuid,
            &player_name,
            &code_hash,
            &minutes,
        ],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn complete(
    client: &mut Client,
    discord_user_id: &str,
    code_hash: &str,
) -> Result<Option<LinkCodeRecord>, StoreError> {
    let mut tx = client.transaction()?;
    let row = tx.query_opt(
        "select player_uuid, player_name from link_codes
         where code_hash = $1 and consumed_at is null and expires_at > now()
         for update",
        &[&code_hash],
    )?;
    let Some(row) = row else {
        tx.commit()?;
        return Ok(None);
    };
    let player_uuid = row.get(0);
    let player_name = row.get(1);
    upsert_verified_tx(&mut tx, discord_user_id, player_uuid)?;
    tx.execute(
        "update link_codes set consumed_at = now() where code_hash = $1",
        &[&code_hash],
    )?;
    tx.commit()?;
    Ok(Some(LinkCodeRecord {
        player_uuid,
        player_name,
    }))
}

pub fn remove_player(client: &mut Client, player_uuid: Uuid) -> Result<bool, StoreError> {
    Ok(client.execute(
        "update discord_account_links set verification_state = 'revoked', revoked_at = now()
         where minecraft_uuid = $1 and revoked_at is null",
        &[&player_uuid],
    )? > 0)
}

pub fn remove_discord(client: &mut Client, discord_user_id: &str) -> Result<bool, StoreError> {
    revoke(client, discord_user_id)
}

pub fn find_by_discord(
    client: &mut Client,
    discord_user_id: &str,
) -> Result<Option<DiscordLinkRecord>, StoreError> {
    get(client, discord_user_id)
}

pub fn upsert_pending(
    client: &mut Client,
    discord_user_id: &str,
    minecraft_uuid: Uuid,
) -> Result<(), StoreError> {
    let metadata = serde_json::Value::Object(Default::default());
    client.execute(
        "insert into discord_account_links (discord_user_id, minecraft_uuid, verification_state, metadata)
         values ($1, $2, 'pending', $3)
         on conflict (discord_user_id) do update set
         minecraft_uuid = excluded.minecraft_uuid,
         verification_state = 'pending', verified_at = null, revoked_at = null",
        &[&discord_user_id, &minecraft_uuid, &metadata],
    )?;
    Ok(())
}

pub fn verify(client: &mut Client, discord_user_id: &str) -> Result<bool, StoreError> {
    Ok(client.execute(
        "update discord_account_links set verification_state = 'verified', verified_at = now()
         where discord_user_id = $1 and revoked_at is null",
        &[&discord_user_id],
    )? == 1)
}

pub fn revoke(client: &mut Client, discord_user_id: &str) -> Result<bool, StoreError> {
    Ok(client.execute(
        "update discord_account_links set verification_state = 'revoked', revoked_at = now()
         where discord_user_id = $1 and revoked_at is null",
        &[&discord_user_id],
    )? == 1)
}

pub fn get(
    client: &mut Client,
    discord_user_id: &str,
) -> Result<Option<DiscordLinkRecord>, StoreError> {
    let row = client.query_opt(
        "select discord_user_id, minecraft_uuid, verification_state, verified_at is not null, revoked_at is not null
         from discord_account_links where discord_user_id = $1",
        &[&discord_user_id],
    )?;
    Ok(row.map(|row| DiscordLinkRecord {
        discord_user_id: row.get(0),
        minecraft_uuid: row.get(1),
        verification_state: row.get(2),
        verified: row.get(3),
        revoked: row.get(4),
    }))
}

fn upsert_verified_tx(
    tx: &mut postgres::Transaction<'_>,
    discord_user_id: &str,
    minecraft_uuid: Uuid,
) -> Result<(), StoreError> {
    let metadata = serde_json::Value::Object(Default::default());
    tx.execute(
        "insert into discord_account_links
         (discord_user_id, minecraft_uuid, verification_state, verified_at, metadata)
         values ($1, $2, 'verified', now(), $3)
         on conflict (discord_user_id) do update set
         minecraft_uuid = excluded.minecraft_uuid,
         verification_state = 'verified', verified_at = now(), revoked_at = null",
        &[&discord_user_id, &minecraft_uuid, &metadata],
    )?;
    Ok(())
}
