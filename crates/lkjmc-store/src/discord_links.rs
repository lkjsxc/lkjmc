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
