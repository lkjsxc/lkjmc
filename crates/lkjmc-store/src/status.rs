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
    Ok(StatusCounts {
        instances: count(client, "instances")?,
        active_sessions: active_sessions(client)?,
        jar_assets: count(client, "jar_assets")?,
        presence_records: count(client, "instance_presence")?,
    })
}

fn count(client: &mut Client, table: &str) -> Result<i64, StoreError> {
    let query = format!("select count(*)::bigint from {table}");
    let row = client.query_one(&query, &[])?;
    Ok(row.get(0))
}

fn active_sessions(client: &mut Client) -> Result<i64, StoreError> {
    let row = client.query_one(
        "select count(*)::bigint from player_sessions where left_at is null",
        &[],
    )?;
    Ok(row.get(0))
}
