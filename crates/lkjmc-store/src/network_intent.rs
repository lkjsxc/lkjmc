use postgres::{Client, Transaction};
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

mod attempts;
pub use attempts::*;

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
    pub effect_phase: String,
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
    let desired = insert_desired(&mut tx, authored_revision, digest, intent, correlation)?;
    tx.commit()?;
    Ok(desired)
}

pub fn record_desired_with_attempt(
    client: &mut Client,
    authored_revision: i64,
    digest: &str,
    intent: &Value,
    correlation: &str,
) -> Result<(DesiredIntent, ApplyAttempt), StoreError> {
    let mut tx = client.transaction()?;
    let desired = insert_desired(&mut tx, authored_revision, digest, intent, correlation)?;
    let attempt = attempts::insert_attempt(&mut tx, desired.revision, correlation)?;
    tx.commit()?;
    Ok((desired, attempt))
}

pub fn desired_by_revision(
    client: &mut Client,
    revision: i64,
) -> Result<Option<DesiredIntent>, StoreError> {
    Ok(client
        .query_opt(
            "select revision, authored_revision, intent_digest, intent, correlation
             from network_intents where revision = $1",
            &[&revision],
        )?
        .map(desired_from_row))
}

pub fn latest_desired(client: &mut Client) -> Result<Option<DesiredIntent>, StoreError> {
    Ok(client
        .query_opt(
            "select revision, authored_revision, intent_digest, intent, correlation
             from network_intents order by revision desc limit 1",
            &[],
        )?
        .map(desired_from_row))
}

fn insert_desired(
    tx: &mut Transaction<'_>,
    authored_revision: i64,
    digest: &str,
    intent: &Value,
    correlation: &str,
) -> Result<DesiredIntent, StoreError> {
    let row = tx.query_opt(
        "insert into network_intents (authored_revision, intent_digest, intent, correlation)
         values ($1, $2, $3, $4) on conflict (correlation) do nothing
         returning revision, authored_revision, intent_digest, intent, correlation",
        &[&authored_revision, &digest, intent, &correlation],
    )?;
    let desired = match row {
        Some(row) => desired_from_row(row),
        None => desired_by_correlation(tx, correlation)?.ok_or_else(|| {
            StoreError::invalid_state("network intent correlation conflict disappeared")
        })?,
    };
    if desired.authored_revision != authored_revision || desired.intent_digest != digest {
        return Err(StoreError::invalid_state(
            "network intent correlation already owns different intent",
        ));
    }
    Ok(desired)
}

fn desired_by_correlation(
    tx: &mut Transaction<'_>,
    correlation: &str,
) -> Result<Option<DesiredIntent>, StoreError> {
    Ok(tx
        .query_opt(
            "select revision, authored_revision, intent_digest, intent, correlation
             from network_intents where correlation = $1",
            &[&correlation],
        )?
        .map(desired_from_row))
}

fn desired_from_row(row: postgres::Row) -> DesiredIntent {
    DesiredIntent {
        revision: row.get(0),
        authored_revision: row.get(1),
        intent_digest: row.get(2),
        intent: row.get(3),
        correlation: row.get(4),
    }
}
