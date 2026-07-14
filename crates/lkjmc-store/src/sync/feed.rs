use crate::error::StoreError;
use postgres::{Client, IsolationLevel};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    pub feed_revision: i64,
    pub domain: String,
    pub key: String,
    pub domain_revision: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeedResult {
    Changes {
        changes: Vec<Change>,
        cursor: i64,
        active_floor: Option<i64>,
    },
    ReloadRequired {
        cursor: i64,
        active_floor: Option<i64>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionResult {
    pub archived: u64,
    pub deleted: u64,
}

pub fn changes_after(
    client: &mut Client,
    after: i64,
    limit: i64,
) -> Result<FeedResult, StoreError> {
    if after < 0 || !(1..=128).contains(&limit) {
        return Err(StoreError::invalid_state("invalid sync cursor or limit"));
    }
    let mut tx = client
        .build_transaction()
        .isolation_level(IsolationLevel::RepeatableRead)
        .read_only(true)
        .start()?;
    let active_floor: Option<i64> = tx
        .query_one("select min(feed_revision) from sync_change_feed", &[])?
        .get(0);
    let issued: i64 = tx
        .query_one(
            "select case when is_called then last_value else 0 end
             from sync_change_feed_feed_revision_seq",
            &[],
        )?
        .get(0);
    if reload_required(after, active_floor, issued) {
        tx.commit()?;
        return Ok(FeedResult::ReloadRequired {
            cursor: issued,
            active_floor,
        });
    }
    let rows = tx.query(
        "select feed_revision, domain, key, domain_revision
         from sync_change_feed where feed_revision > $1
         order by feed_revision limit $2",
        &[&after, &limit],
    )?;
    let changes = rows
        .into_iter()
        .map(|row| Change {
            feed_revision: row.get(0),
            domain: row.get(1),
            key: row.get(2),
            domain_revision: row.get(3),
        })
        .collect::<Vec<_>>();
    let cursor = changes.last().map_or(after, |change| change.feed_revision);
    tx.commit()?;
    Ok(FeedResult::Changes {
        changes,
        cursor,
        active_floor,
    })
}

pub fn run_retention(client: &mut Client) -> Result<RetentionResult, StoreError> {
    let mut tx = client.transaction()?;
    let policy = tx.query_one(
        "select active_days,archive_days,batch_size from sync_retention_policy where singleton",
        &[],
    )?;
    let active_days: i32 = policy.get(0);
    let archive_days: i32 = policy.get(1);
    let batch_size: i32 = policy.get(2);
    let active = revisions(
        &mut tx,
        "select feed_revision from sync_change_feed
         where created_at < now() - make_interval(days => $1::integer)
         order by feed_revision for update skip locked limit $2::integer",
        active_days,
        batch_size,
    )?;
    let archived = tx.execute(
        "insert into sync_change_archive(feed_revision,writer_xid,domain,key,
         domain_revision,created_at) select feed_revision,writer_xid,domain,key,
         domain_revision,created_at from sync_change_feed where feed_revision = any($1)
         on conflict do nothing",
        &[&active],
    )?;
    tx.execute(
        "delete from sync_change_feed f where feed_revision = any($1)
         and exists (select 1 from sync_change_archive a
         where a.feed_revision=f.feed_revision)",
        &[&active],
    )?;
    let expired = revisions(
        &mut tx,
        "select feed_revision from sync_change_archive
         where created_at < now() - make_interval(days => $1::integer)
         order by feed_revision for update skip locked limit $2::integer",
        archive_days,
        batch_size,
    )?;
    let deleted = tx.execute(
        "delete from sync_change_archive where feed_revision = any($1)",
        &[&expired],
    )?;
    tx.commit()?;
    Ok(RetentionResult { archived, deleted })
}

fn revisions(
    client: &mut impl postgres::GenericClient,
    query: &str,
    days: i32,
    limit: i32,
) -> Result<Vec<i64>, StoreError> {
    Ok(client
        .query(query, &[&days, &limit])?
        .into_iter()
        .map(|row| row.get(0))
        .collect())
}

fn reload_required(after: i64, floor: Option<i64>, issued: i64) -> bool {
    match floor {
        Some(value) => after < value.saturating_sub(1),
        None => after < issued,
    }
}

#[cfg(test)]
mod tests {
    use super::reload_required;

    #[test]
    fn cursor_immediately_before_floor_can_read_floor() {
        assert!(!reload_required(9, Some(10), 12));
        assert!(reload_required(8, Some(10), 12));
        assert!(reload_required(11, None, 12));
        assert!(!reload_required(12, None, 12));
    }
}
