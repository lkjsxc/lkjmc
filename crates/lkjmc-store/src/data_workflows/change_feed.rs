use postgres::{Client, GenericClient};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Clone, Debug, PartialEq)]
pub struct ChangeRecord {
    pub feed_revision: i64,
    pub aggregate_kind: String,
    pub aggregate_id: Uuid,
    pub aggregate_revision: i64,
    pub correlation_id: Uuid,
    pub state: String,
    pub fact: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResumeResult {
    Changes(Vec<ChangeRecord>),
    ReloadRequired { active_floor: Option<i64> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionResult {
    pub archived: u64,
    pub deleted_active: u64,
    pub deleted_archive: u64,
}

pub(crate) fn append(
    client: &mut impl GenericClient,
    kind: &str,
    id: Uuid,
    revision: i64,
    correlation: Uuid,
    state: &str,
) -> Result<i64, StoreError> {
    let row = client.query_one(
        "insert into workflow_change_feed
         (aggregate_kind, aggregate_id, aggregate_revision, correlation_id, state, fact)
         values ($1, $2, $3, $4, $5, $6) returning feed_revision",
        &[&kind, &id, &revision, &correlation, &state, &json!({})],
    )?;
    Ok(row.get(0))
}

pub fn changes_after(
    client: &mut Client,
    after: i64,
    limit: i64,
) -> Result<ResumeResult, StoreError> {
    if after < 0 || !(1..=1000).contains(&limit) {
        return Err(StoreError::invalid_state(
            "invalid change feed cursor or limit",
        ));
    }
    let active_floor = retained_floor(client)?;
    let issued: i64 = client
        .query_one(
            "select case when is_called then last_value else 0 end
             from workflow_change_feed_feed_revision_seq",
            &[],
        )?
        .get(0);
    if reload_required(after, active_floor, issued) {
        return Ok(ResumeResult::ReloadRequired { active_floor });
    }
    let rows = client.query(
        "select feed_revision, aggregate_kind, aggregate_id, aggregate_revision,
         correlation_id, state, fact from workflow_change_feed
         where feed_revision > $1 order by feed_revision limit $2",
        &[&after, &limit],
    )?;
    Ok(ResumeResult::Changes(
        rows.into_iter()
            .map(|row| ChangeRecord {
                feed_revision: row.get(0),
                aggregate_kind: row.get(1),
                aggregate_id: row.get(2),
                aggregate_revision: row.get(3),
                correlation_id: row.get(4),
                state: row.get(5),
                fact: row.get(6),
            })
            .collect(),
    ))
}

pub fn retained_floor(client: &mut Client) -> Result<Option<i64>, StoreError> {
    let row = client.query_one("select min(feed_revision) from workflow_change_feed", &[])?;
    Ok(row.get(0))
}

fn reload_required(after: i64, active_floor: Option<i64>, issued: i64) -> bool {
    match active_floor {
        Some(floor) => after < floor,
        None => after < issued,
    }
}

pub fn run_retention(client: &mut Client) -> Result<RetentionResult, StoreError> {
    let mut tx = client.transaction()?;
    let archived = tx.execute(
        "insert into workflow_change_archive
         (feed_revision, aggregate_kind, aggregate_id, aggregate_revision,
          correlation_id, state, fact, created_at)
         select feed_revision, aggregate_kind, aggregate_id, aggregate_revision,
          correlation_id, state, fact, created_at from workflow_change_feed
         where created_at < now() - interval '30 days'
         on conflict (feed_revision) do nothing",
        &[],
    )?;
    let deleted_active = tx.execute(
        "delete from workflow_change_feed where created_at < now() - interval '30 days'",
        &[],
    )?;
    let deleted_archive = tx.execute(
        "delete from workflow_change_archive where created_at < now() - interval '365 days'",
        &[],
    )?;
    tx.commit()?;
    Ok(RetentionResult {
        archived,
        deleted_active,
        deleted_archive,
    })
}

#[cfg(test)]
mod tests {
    use super::reload_required;

    #[test]
    fn resume_decision_is_pure_at_active_archive_and_deleted_boundaries() {
        assert!(reload_required(9, Some(10), 12));
        assert!(!reload_required(10, Some(10), 12));
        assert!(reload_required(11, None, 12));
        assert!(!reload_required(12, None, 12));
        assert!(!reload_required(0, None, 0));
    }
}
