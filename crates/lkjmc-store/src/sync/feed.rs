use crate::error::StoreError;
use postgres::Client;

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
    let mut tx = client.transaction()?;
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
    let deleted = client.execute(
        "delete from sync_change_feed
         where created_at < now() - interval '30 days'",
        &[],
    )?;
    Ok(RetentionResult { deleted })
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
