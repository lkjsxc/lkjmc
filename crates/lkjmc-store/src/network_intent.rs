use postgres::{Client, Transaction};
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq)]
pub struct DesiredIntent {
    pub revision: i64,
    pub authored_revision: i64,
    pub intent_digest: String,
    pub intent: Value,
    pub correlation: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyAttempt {
    pub id: Uuid,
    pub network_revision: i64,
    pub correlation: String,
    pub outcome: String,
    pub diagnostic: Option<String>,
    pub observation: Value,
}

pub fn record_desired(
    client: &mut Client,
    authored_revision: i64,
    digest: &str,
    intent: &Value,
    correlation: &str,
) -> Result<DesiredIntent, StoreError> {
    let mut tx = client.transaction()?;
    let row = tx.query_opt(
        "insert into network_intents (authored_revision, intent_digest, intent, correlation)
         values ($1, $2, $3, $4) on conflict (correlation) do nothing
         returning revision, authored_revision, intent_digest, intent, correlation",
        &[&authored_revision, &digest, intent, &correlation],
    )?;
    let desired = match row {
        Some(row) => desired_from_row(row),
        None => desired_by_correlation(&mut tx, correlation)?.ok_or_else(|| {
            StoreError::invalid_state("network intent correlation conflict disappeared")
        })?,
    };
    if desired.authored_revision != authored_revision || desired.intent_digest != digest {
        return Err(StoreError::invalid_state(
            "network intent correlation already owns different intent",
        ));
    }
    tx.commit()?;
    Ok(desired)
}

pub fn create_attempt(
    client: &mut Client,
    network_revision: i64,
    correlation: &str,
) -> Result<ApplyAttempt, StoreError> {
    let id = Uuid::new_v4();
    let row = client.query_one(
        "insert into network_apply_attempts
         (id, network_revision, correlation, outcome)
         values ($1, $2, $3, 'planned')
         returning id, network_revision, correlation, outcome, diagnostic, observation",
        &[&id, &network_revision, &correlation],
    )?;
    Ok(attempt_from_row(row))
}

pub fn mark_applying(client: &mut Client, id: Uuid) -> Result<(), StoreError> {
    let changed = client.execute(
        "update network_apply_attempts set outcome = 'applying'
         where id = $1 and outcome = 'planned'",
        &[&id],
    )?;
    exactly_one(changed, "network attempt is not planned")
}

pub fn finish_attempt(
    client: &mut Client,
    id: Uuid,
    outcome: &str,
    diagnostic: Option<&str>,
    observation: &Value,
) -> Result<(), StoreError> {
    if !matches!(outcome, "observed" | "failed" | "unsupported" | "no-op") {
        return Err(StoreError::invalid_state("invalid terminal network outcome"));
    }
    let changed = client.execute(
        "update network_apply_attempts
         set outcome = $2, diagnostic = $3, observation = $4, finished_at = now()
         where id = $1 and outcome in ('planned', 'applying')",
        &[&id, &outcome, &diagnostic, observation],
    )?;
    exactly_one(changed, "network attempt is already terminal")
}

pub fn latest_desired(client: &mut Client) -> Result<Option<DesiredIntent>, StoreError> {
    Ok(client.query_opt(
        "select revision, authored_revision, intent_digest, intent, correlation
         from network_intents order by revision desc limit 1",
        &[],
    )?.map(desired_from_row))
}

pub fn attempts_for_revision(client: &mut Client, revision: i64) -> Result<Vec<ApplyAttempt>, StoreError> {
    Ok(client.query(
        "select id, network_revision, correlation, outcome, diagnostic, observation
         from network_apply_attempts where network_revision = $1 order by started_at, id",
        &[&revision],
    )?.into_iter().map(attempt_from_row).collect())
}

fn desired_by_correlation(tx: &mut Transaction<'_>, correlation: &str) -> Result<Option<DesiredIntent>, StoreError> {
    Ok(tx.query_opt(
        "select revision, authored_revision, intent_digest, intent, correlation
         from network_intents where correlation = $1",
        &[&correlation],
    )?.map(desired_from_row))
}

fn desired_from_row(row: postgres::Row) -> DesiredIntent {
    DesiredIntent { revision: row.get(0), authored_revision: row.get(1), intent_digest: row.get(2), intent: row.get(3), correlation: row.get(4) }
}
fn attempt_from_row(row: postgres::Row) -> ApplyAttempt {
    ApplyAttempt { id: row.get(0), network_revision: row.get(1), correlation: row.get(2), outcome: row.get(3), diagnostic: row.get(4), observation: row.get(5) }
}
fn exactly_one(changed: u64, message: &'static str) -> Result<(), StoreError> {
    if changed == 1 { Ok(()) } else { Err(StoreError::invalid_state(message)) }
}
