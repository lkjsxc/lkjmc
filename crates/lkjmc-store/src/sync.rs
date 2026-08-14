mod feed;
mod payload;

use postgres::{Client, IsolationLevel};
use serde_json::Value;

use crate::error::StoreError;

pub use feed::{changes_after, run_retention, Change, FeedResult, RetentionResult};

pub const DOMAINS: &[&str] = &[
    "permissions",
    "claims",
    "profiles",
    "presence",
    "routing",
    "settings",
];

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub domain: String,
    pub key: String,
    pub revision: i64,
    pub generated_at: String,
    pub payload: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SnapshotResult {
    Available(Snapshot),
    Unavailable { reason: String },
}

pub fn snapshot(
    client: &mut Client,
    domain: &str,
    key: &str,
) -> Result<SnapshotResult, StoreError> {
    validate(domain, key)?;
    let mut tx = client
        .build_transaction()
        .isolation_level(IsolationLevel::RepeatableRead)
        .read_only(true)
        .start()?;
    let Some(row) = tx.query_opt(
        "select revision, to_char(transaction_timestamp() at time zone 'UTC',
         'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"')
         from sync_domain_revisions where domain = $1 and key = $2",
        &[&domain, &key],
    )?
    else {
        tx.commit()?;
        return Ok(SnapshotResult::Unavailable {
            reason: "key-not-found".into(),
        });
    };
    let revision = row.get(0);
    let generated_at = row.get(1);
    let payload = payload::read(&mut tx, domain, key)?;
    tx.commit()?;
    Ok(SnapshotResult::Available(Snapshot {
        domain: domain.to_string(),
        key: key.to_string(),
        revision,
        generated_at,
        payload,
    }))
}

fn validate(domain: &str, key: &str) -> Result<(), StoreError> {
    if !DOMAINS.contains(&domain) || key.is_empty() || key.len() > 256 {
        return Err(StoreError::invalid_state("invalid sync domain or key"));
    }
    Ok(())
}
