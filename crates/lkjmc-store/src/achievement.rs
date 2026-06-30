mod support;

use postgres::{Client, GenericClient};
use serde_json::json;
use uuid::Uuid;

use crate::error::StoreError;
use support::{claim_row, progress_definition, progress_from_row, upsert_definition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementProgressRecord {
    pub id: String,
    pub title_key: String,
    pub description_key: String,
    pub category: String,
    pub icon_material: String,
    pub current: i64,
    pub required: i64,
    pub claimed: bool,
    pub reward_claimed: bool,
}

pub fn seed_defaults(client: &mut impl GenericClient) -> Result<(), StoreError> {
    lkjmc_core::achievement::validate(lkjmc_core::achievement::DEFAULT_ACHIEVEMENTS)
        .map_err(StoreError::invalid_state)?;
    for item in lkjmc_core::achievement::DEFAULT_ACHIEVEMENTS {
        upsert_definition(client, item)?;
    }
    Ok(())
}

pub fn grant(
    client: &mut Client,
    player_uuid: Uuid,
    achievement_id: &str,
    title_key: &str,
) -> Result<(), StoreError> {
    let config = json!({
        "descriptionKey": title_key,
        "category": "legacy",
        "iconMaterial": "EMERALD",
        "criteria": {"kind":"legacy", "threshold":1},
        "reward": {"points":0}
    });
    client.execute(
        "insert into achievements (id, title_key, config) values ($1, $2, $3)
         on conflict (id) do update set title_key = excluded.title_key, config = excluded.config",
        &[&achievement_id, &title_key, &config],
    )?;
    claim_row(client, player_uuid, achievement_id, 1, true)
}

pub fn apply_event(
    client: &mut Client,
    player_uuid: Uuid,
    criteria_kind: &str,
    amount: i64,
    correlation_id: Option<Uuid>,
) -> Result<Vec<String>, StoreError> {
    seed_defaults(client)?;
    let mut claimed = Vec::new();
    for definition in lkjmc_core::achievement::by_criteria(criteria_kind) {
        if progress_definition(client, player_uuid, definition, amount, correlation_id)? {
            claimed.push(definition.id.to_string());
        }
    }
    Ok(claimed)
}

pub fn list_progress(
    client: &mut Client,
    player_uuid: Uuid,
) -> Result<Vec<AchievementProgressRecord>, StoreError> {
    seed_defaults(client)?;
    let rows = client.query(
        "select a.id, a.title_key, a.config, coalesce(pa.progress, '{}'::jsonb),
         coalesce(pa.claimed, false) from achievements a
         left join player_achievements pa on pa.achievement_id = a.id and pa.player_uuid = $1
         order by a.id",
        &[&player_uuid],
    )?;
    Ok(rows.into_iter().filter_map(progress_from_row).collect())
}

pub fn list_claimed(
    client: &mut Client,
    player_uuid: Uuid,
) -> Result<Vec<AchievementProgressRecord>, StoreError> {
    Ok(list_progress(client, player_uuid)?
        .into_iter()
        .filter(|row| row.claimed)
        .collect())
}
