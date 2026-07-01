use postgres::{Client, GenericClient};
use serde_json::{json, Value};
use uuid::Uuid;

use super::AchievementProgressRecord;
use crate::error::StoreError;

pub(super) fn upsert_definition(
    client: &mut impl GenericClient,
    item: &lkjmc_core::achievement::AchievementDefinition,
) -> Result<(), StoreError> {
    let config = json!({
        "descriptionKey": item.description_key,
        "category": item.category,
        "iconMaterial": item.icon_material,
        "criteria": {"kind": item.criteria_kind, "threshold": item.threshold},
        "reward": {"points": item.reward_points},
        "hidden": item.hidden,
        "repeatable": item.repeatable
    });
    client.execute(
        "insert into achievements (id, title_key, config) values ($1, $2, $3)
         on conflict (id) do update set title_key = excluded.title_key, config = excluded.config",
        &[&item.id, &item.title_key, &config],
    )?;
    Ok(())
}

pub(super) fn progress_definition(
    client: &mut Client,
    player_uuid: Uuid,
    definition: &lkjmc_core::achievement::AchievementDefinition,
    amount: i64,
    correlation_id: Option<Uuid>,
) -> Result<bool, StoreError> {
    let row = client.query_opt(
        "select progress, claimed from player_achievements
         where player_uuid = $1 and achievement_id = $2",
        &[&player_uuid, &definition.id],
    )?;
    let (progress, already_claimed) = row
        .map(|row| (row.get::<_, Value>(0), row.get::<_, bool>(1)))
        .unwrap_or_else(|| (json!({}), false));
    if seen(&progress, correlation_id) {
        return Ok(false);
    }
    let current = progress
        .get("current")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .saturating_add(amount.max(0));
    let claimed = already_claimed || current >= definition.threshold;
    claim_row(client, player_uuid, definition.id, current, claimed)?;
    let next = with_seen(current, &progress, correlation_id);
    client.execute(
        "update player_achievements set progress = $3 where player_uuid = $1 and achievement_id = $2",
        &[&player_uuid, &definition.id, &next],
    )?;
    Ok(claimed && !already_claimed)
}

pub(super) fn claim_row(
    client: &mut impl GenericClient,
    player_uuid: Uuid,
    achievement_id: &str,
    current: i64,
    claimed: bool,
) -> Result<(), StoreError> {
    let progress = json!({"current": current});
    client.execute(
        "insert into player_achievements (player_uuid, achievement_id, progress, claimed)
         values ($1, $2, $3, $4)
         on conflict (player_uuid, achievement_id) do update set
         progress = excluded.progress, claimed = excluded.claimed, updated_at = now()",
        &[&player_uuid, &achievement_id, &progress, &claimed],
    )?;
    Ok(())
}

pub(super) fn progress_from_row(row: postgres::Row) -> Option<AchievementProgressRecord> {
    let config: Value = row.get(2);
    let progress: Value = row.get(3);
    let required = config
        .pointer("/criteria/threshold")
        .and_then(Value::as_i64)
        .unwrap_or(1);
    let current = progress.get("current").and_then(Value::as_i64).unwrap_or(0);
    let hidden = config
        .get("hidden")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let claimed: bool = row.get(4);
    let reward_claimed: bool = row.get(5);
    if hidden && current == 0 && !claimed {
        return None;
    }
    Some(AchievementProgressRecord {
        id: row.get(0),
        title_key: row.get(1),
        description_key: config.get("descriptionKey")?.as_str()?.to_string(),
        category: config.get("category")?.as_str()?.to_string(),
        icon_material: config.get("iconMaterial")?.as_str()?.to_string(),
        current,
        required,
        claimed,
        reward_claimed,
    })
}

pub(super) fn deliver_mail_reward(
    client: &mut impl GenericClient,
    player_uuid: Uuid,
    achievement_id: &str,
    body: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_mail_messages (id, recipient_uuid, sender_uuid, sender_name, body)
         values ($1, $2, null, 'lkjmc achievements', $3)",
        &[&Uuid::new_v4(), &player_uuid, &body],
    )?;
    client.execute(
        "insert into achievement_reward_claims (id, player_uuid, achievement_id, reward_id, reward_kind)
         values ($1, $2, $3, 'mail', 'mail') on conflict (player_uuid, achievement_id, reward_id) do nothing",
        &[&Uuid::new_v4(), &player_uuid, &achievement_id],
    )?;
    Ok(())
}

pub(super) fn reward_id(player_uuid: Uuid, achievement_id: &str) -> Uuid {
    Uuid::new_v5(
        &player_uuid,
        format!("achievement.reward.claim:{achievement_id}:default").as_bytes(),
    )
}

fn seen(progress: &Value, correlation_id: Option<Uuid>) -> bool {
    correlation_id.is_some_and(|id| {
        progress
            .get("seen")
            .and_then(Value::as_array)
            .is_some_and(|seen| {
                seen.iter()
                    .any(|value| value.as_str() == Some(&id.to_string()))
            })
    })
}

fn with_seen(current: i64, progress: &Value, correlation_id: Option<Uuid>) -> Value {
    let mut seen = progress
        .get("seen")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(id) = correlation_id {
        seen.push(Value::String(id.to_string()));
    }
    json!({"current": current, "seen": seen})
}
