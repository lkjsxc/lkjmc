use postgres::GenericClient;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

pub fn insert_identity(
    client: &mut impl GenericClient,
    player_uuid: Uuid,
    name: &str,
) -> Result<(), StoreError> {
    ensure_identity(client, player_uuid, Some(name))
}

pub fn ensure_identity(
    client: &mut impl GenericClient,
    player_uuid: Uuid,
    name: Option<&str>,
) -> Result<(), StoreError> {
    let name = name.map(str::trim).filter(|value| !value.is_empty());
    let current_name = name.unwrap_or("unknown");
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into player_identities (player_uuid, current_name, metadata)
         values ($1, $2, $3)
         on conflict (player_uuid) do update set
         current_name = case when $4 then excluded.current_name else player_identities.current_name end,
         last_seen_at = now()",
        &[&player_uuid, &current_name, &metadata, &name.is_some()],
    )?;
    Ok(())
}
