use postgres::Client;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusCounts {
    pub instances: i64,
    pub active_sessions: i64,
    pub jar_assets: i64,
    pub presence_records: i64,
}

pub fn counts(client: &mut Client) -> Result<StatusCounts, StoreError> {
    let row = client.query_one(
        "select
           (select count(*)::bigint from instances),
           (select count(*)::bigint from player_sessions where left_at is null),
           (select count(*)::bigint from jar_assets),
           (select count(*)::bigint from instance_presence)",
        &[],
    )?;
    Ok(StatusCounts {
        instances: row.get(0),
        active_sessions: row.get(1),
        jar_assets: row.get(2),
        presence_records: row.get(3),
    })
}
