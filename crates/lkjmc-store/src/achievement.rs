mod event;
mod support;

pub use event::{apply_event, apply_event_for_player, AchievementEventOutcome};

use postgres::{Client, GenericClient};
use serde_json::json;
use uuid::Uuid;

use crate::error::StoreError;
use support::{claim_row, deliver_mail_reward, progress_from_row, reward_id, upsert_definition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementClaimResult {
    pub achievement_id: String,
    pub reward_claimed: bool,
    pub already_claimed: bool,
    pub points: i64,
    pub mail_delivered: bool,
    pub ledger_id: Option<Uuid>,
}

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

pub fn list_progress(
    client: &mut Client,
    player_uuid: Uuid,
) -> Result<Vec<AchievementProgressRecord>, StoreError> {
    seed_defaults(client)?;
    let rows = client.query(
        "select a.id, a.title_key, a.config, coalesce(pa.progress, '{}'::jsonb),
         coalesce(pa.claimed, false), coalesce(pa.reward_claimed, false)
         from achievements a
         left join player_achievements pa on pa.achievement_id = a.id and pa.player_uuid = $1
         order by a.id",
        &[&player_uuid],
    )?;
    Ok(rows.into_iter().filter_map(progress_from_row).collect())
}

pub fn claim_reward(
    client: &mut Client,
    player_uuid: Uuid,
    achievement_id: &str,
) -> Result<AchievementClaimResult, StoreError> {
    seed_defaults(client)?;
    let mut tx = client.transaction()?;
    let row = tx
        .query_opt(
            "select a.config, coalesce(pa.claimed, false), coalesce(pa.reward_claimed, false)
             from achievements a
             left join player_achievements pa on pa.achievement_id = a.id and pa.player_uuid = $1
             where a.id = $2",
            &[&player_uuid, &achievement_id],
        )?
        .ok_or_else(|| StoreError::invalid_state("achievement not found"))?;
    let config: serde_json::Value = row.get(0);
    let claimed: bool = row.get(1);
    let already_claimed: bool = row.get(2);
    if !claimed {
        return Err(StoreError::invalid_state("achievement is not complete"));
    }
    if already_claimed {
        tx.commit()?;
        return Ok(AchievementClaimResult {
            achievement_id: achievement_id.to_string(),
            reward_claimed: true,
            already_claimed: true,
            points: 0,
            mail_delivered: false,
            ledger_id: None,
        });
    }
    let points = config
        .pointer("/reward/points")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);
    let ledger_id = if points > 0 {
        Some(crate::points::grant_with_correlation(
            &mut tx,
            player_uuid,
            points,
            "achievement.reward.claim",
            Some(reward_id(player_uuid, achievement_id)),
        )?)
    } else {
        None
    };
    let mail_body = config
        .pointer("/reward/mail/body")
        .and_then(serde_json::Value::as_str);
    if let Some(body) = mail_body {
        deliver_mail_reward(&mut tx, player_uuid, achievement_id, body)?;
    }
    tx.execute(
        "update player_achievements set reward_claimed = true, reward_claimed_at = now(), updated_at = now()
         where player_uuid = $1 and achievement_id = $2",
        &[&player_uuid, &achievement_id],
    )?;
    tx.execute(
        "insert into achievement_reward_claims
         (id, player_uuid, achievement_id, reward_id, reward_kind, points_delta, ledger_id)
         values ($1, $2, $3, 'default', 'points', $4, $5)
         on conflict (player_uuid, achievement_id, reward_id) do nothing",
        &[
            &Uuid::new_v4(),
            &player_uuid,
            &achievement_id,
            &points,
            &ledger_id,
        ],
    )?;
    tx.commit()?;
    Ok(AchievementClaimResult {
        achievement_id: achievement_id.to_string(),
        reward_claimed: true,
        already_claimed: false,
        points,
        mail_delivered: mail_body.is_some(),
        ledger_id,
    })
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
