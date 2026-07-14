use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use super::ApplyAttempt;
use crate::error::StoreError;

pub fn create_attempt(
    client: &mut Client,
    network_revision: i64,
    correlation: &str,
) -> Result<ApplyAttempt, StoreError> {
    insert_attempt(client, network_revision, correlation)
}

pub fn mark_applying(client: &mut Client, id: Uuid) -> Result<(), StoreError> {
    let changed = client.execute(
        "update network_apply_attempts set outcome = 'applying'
         where id = $1 and outcome = 'planned'",
        &[&id],
    )?;
    exactly_one(changed, "network attempt is not planned")
}

pub fn mark_effect_phase(client: &mut Client, id: Uuid, phase: &str) -> Result<(), StoreError> {
    if !matches!(phase, "configuration" | "runtime" | "observation") {
        return Err(StoreError::invalid_state("invalid network effect phase"));
    }
    let changed = client.execute(
        "update network_apply_attempts set effect_phase = $2
         where id = $1 and outcome = 'applying'
           and array_position(array['none','configuration','runtime','observation'], effect_phase)
            <= array_position(array['none','configuration','runtime','observation'], $2)",
        &[&id, &phase],
    )?;
    exactly_one(changed, "network attempt is not applying")
}

pub fn finish_attempt(
    client: &mut Client,
    id: Uuid,
    outcome: &str,
    diagnostic: Option<&str>,
    observation: &Value,
) -> Result<(), StoreError> {
    if !matches!(
        outcome,
        "observed" | "failed" | "unknown" | "unsupported" | "no-op"
    ) {
        return Err(StoreError::invalid_state(
            "invalid terminal network outcome",
        ));
    }
    let changed = client.execute(
        "update network_apply_attempts
         set outcome = $2, diagnostic = $3, observation = $4, finished_at = now()
         where id = $1 and outcome in ('planned', 'applying')",
        &[&id, &outcome, &diagnostic, observation],
    )?;
    exactly_one(changed, "network attempt is already terminal")
}

pub fn attempt(client: &mut Client, id: Uuid) -> Result<Option<ApplyAttempt>, StoreError> {
    Ok(client
        .query_opt(
            "select id, network_revision, correlation, outcome, effect_phase,
                    diagnostic, observation
             from network_apply_attempts where id = $1",
            &[&id],
        )?
        .map(attempt_from_row))
}

pub fn recovery_candidates(client: &mut Client) -> Result<Vec<ApplyAttempt>, StoreError> {
    Ok(client
        .query(
            "select id, network_revision, correlation, outcome, effect_phase,
                    diagnostic, observation
             from network_apply_attempts
             where outcome in ('planned', 'applying')
                or (outcome = 'unknown' and
                    coalesce(observation->>'recoveryComplete', 'false') <> 'true')
             order by started_at, id",
            &[],
        )?
        .into_iter()
        .map(attempt_from_row)
        .collect())
}

pub fn complete_unknown(
    client: &mut Client,
    id: Uuid,
    observation: &Value,
) -> Result<(), StoreError> {
    let changed = client.execute(
        "update network_apply_attempts set observation = $2, finished_at = now()
         where id = $1 and outcome = 'unknown'",
        &[&id, observation],
    )?;
    exactly_one(changed, "network attempt is not unknown")
}

pub fn attempts_for_correlation(
    client: &mut Client,
    correlation: &str,
) -> Result<Vec<ApplyAttempt>, StoreError> {
    Ok(client
        .query(
            "select id, network_revision, correlation, outcome, effect_phase,
                    diagnostic, observation
             from network_apply_attempts where correlation = $1 order by started_at, id",
            &[&correlation],
        )?
        .into_iter()
        .map(attempt_from_row)
        .collect())
}

pub fn attempts_for_revision(
    client: &mut Client,
    revision: i64,
) -> Result<Vec<ApplyAttempt>, StoreError> {
    let rows = client.query(
        "select id, network_revision, correlation, outcome, effect_phase,
                diagnostic, observation
         from network_apply_attempts where network_revision = $1 order by started_at, id",
        &[&revision],
    )?;
    Ok(rows.into_iter().map(attempt_from_row).collect())
}

pub(super) fn insert_attempt(
    client: &mut impl postgres::GenericClient,
    network_revision: i64,
    correlation: &str,
) -> Result<ApplyAttempt, StoreError> {
    let id = Uuid::new_v4();
    let row = client.query_one(
        "insert into network_apply_attempts
         (id, network_revision, correlation, outcome)
         values ($1, $2, $3, 'planned')
         returning id, network_revision, correlation, outcome, effect_phase,
                   diagnostic, observation",
        &[&id, &network_revision, &correlation],
    )?;
    Ok(attempt_from_row(row))
}

fn attempt_from_row(row: postgres::Row) -> ApplyAttempt {
    ApplyAttempt {
        id: row.get(0),
        network_revision: row.get(1),
        correlation: row.get(2),
        outcome: row.get(3),
        effect_phase: row.get(4),
        diagnostic: row.get(5),
        observation: row.get(6),
    }
}

fn exactly_one(changed: u64, message: &'static str) -> Result<(), StoreError> {
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::invalid_state(message))
    }
}
