use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitDefinition {
    pub id: String,
    pub title_key: String,
    pub reward_points: i64,
    pub cooldown_hours: i32,
}

pub fn upsert(
    client: &mut Client,
    id: &str,
    title_key: &str,
    reward_points: i64,
    cooldown_hours: i32,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into kit_definitions (id, title_key, reward_points, cooldown_hours, metadata)
         values ($1, $2, $3, $4, $5)
         on conflict (id) do update set title_key = excluded.title_key,
         reward_points = excluded.reward_points,
         cooldown_hours = excluded.cooldown_hours,
         metadata = excluded.metadata,
         updated_at = now()",
        &[&id, &title_key, &reward_points, &cooldown_hours, &metadata],
    )?;
    Ok(())
}

pub fn list(client: &mut Client) -> Result<Vec<KitDefinition>, StoreError> {
    let rows = client.query(
        "select id, title_key, reward_points, cooldown_hours from kit_definitions order by id",
        &[],
    )?;
    Ok(rows.into_iter().map(kit_from_row).collect())
}

pub fn get(client: &mut Client, id: &str) -> Result<Option<KitDefinition>, StoreError> {
    let row = client.query_opt(
        "select id, title_key, reward_points, cooldown_hours from kit_definitions where id = $1",
        &[&id],
    )?;
    Ok(row.map(kit_from_row))
}

pub fn claim(
    client: &mut Client,
    player_uuid: Uuid,
    kit: &KitDefinition,
) -> Result<bool, StoreError> {
    crate::player::ensure_identity(client, player_uuid, None)?;
    let rows = client.execute(
        "insert into player_kit_claims (id, player_uuid, kit_id, reward_points)
         select $1, $2, $3, $5 where not exists (
             select 1 from player_kit_claims
             where player_uuid = $2 and kit_id = $3
             and claimed_at > now() - ($4::integer * interval '1 hour')
         )",
        &[
            &Uuid::new_v4(),
            &player_uuid,
            &kit.id,
            &kit.cooldown_hours,
            &kit.reward_points,
        ],
    )?;
    Ok(rows == 1)
}

fn kit_from_row(row: postgres::Row) -> KitDefinition {
    KitDefinition {
        id: row.get(0),
        title_key: row.get(1),
        reward_points: row.get(2),
        cooldown_hours: row.get(3),
    }
}
