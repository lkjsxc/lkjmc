use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerReport {
    pub id: Uuid,
    pub reporter_uuid: Uuid,
    pub target_uuid: Uuid,
    pub server_id: String,
    pub reason: String,
    pub status: String,
}

pub fn create(
    client: &mut Client,
    id: Uuid,
    reporter_uuid: Uuid,
    target_uuid: Uuid,
    server_id: &str,
    reason: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_reports (id, reporter_uuid, target_uuid, server_id, reason)
         values ($1, $2, $3, $4, $5)",
        &[&id, &reporter_uuid, &target_uuid, &server_id, &reason],
    )?;
    Ok(())
}

pub fn open(client: &mut Client, limit: i64) -> Result<Vec<PlayerReport>, StoreError> {
    let rows = client.query(
        "select id, reporter_uuid, target_uuid, server_id, reason, status
         from player_reports where status = 'open' order by created_at desc limit $1",
        &[&limit],
    )?;
    Ok(rows.into_iter().map(report_from_row).collect())
}

pub fn close(client: &mut Client, id: Uuid, status: &str) -> Result<u64, StoreError> {
    Ok(client.execute(
        "update player_reports set status = $2, resolved_at = now()
         where id = $1 and status = 'open' and $2 in ('resolved', 'dismissed')",
        &[&id, &status],
    )?)
}

fn report_from_row(row: postgres::Row) -> PlayerReport {
    PlayerReport {
        id: row.get(0),
        reporter_uuid: row.get(1),
        target_uuid: row.get(2),
        server_id: row.get(3),
        reason: row.get(4),
        status: row.get(5),
    }
}
