use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementRecord {
    pub id: String,
    pub title_key: String,
    pub claimed: bool,
}

pub fn grant(
    client: &mut Client,
    player_uuid: Uuid,
    achievement_id: &str,
    title_key: &str,
) -> Result<(), StoreError> {
    let config = Value::Object(Default::default());
    client.execute(
        "insert into achievements (id, title_key, config) values ($1, $2, $3)
         on conflict (id) do update set title_key = excluded.title_key",
        &[&achievement_id, &title_key, &config],
    )?;
    client.execute(
        "insert into player_achievements (player_uuid, achievement_id, progress, claimed)
         values ($1, $2, '{}'::jsonb, true)
         on conflict (player_uuid, achievement_id) do update set
         claimed = true,
         updated_at = now()",
        &[&player_uuid, &achievement_id],
    )?;
    Ok(())
}

pub fn list_claimed(
    client: &mut Client,
    player_uuid: Uuid,
) -> Result<Vec<AchievementRecord>, StoreError> {
    let rows = client.query(
        "select a.id, a.title_key, pa.claimed
         from player_achievements pa join achievements a on a.id = pa.achievement_id
         where pa.player_uuid = $1 and pa.claimed = true order by a.id",
        &[&player_uuid],
    )?;
    Ok(rows
        .into_iter()
        .map(|row| AchievementRecord {
            id: row.get(0),
            title_key: row.get(1),
            claimed: row.get(2),
        })
        .collect())
}
