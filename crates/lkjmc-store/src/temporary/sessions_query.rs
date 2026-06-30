use postgres::{GenericClient, Row};
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureSessionSummary {
    pub id: Uuid,
    pub adventure_kind: String,
    pub buyer_uuid: Uuid,
    pub temporary_instance_id: String,
    pub state: String,
    pub points_cost: i64,
}

pub fn list_sessions(
    client: &mut impl GenericClient,
    limit: i64,
) -> Result<Vec<AdventureSessionSummary>, StoreError> {
    let rows = client.query(
        "select id, adventure_kind, buyer_uuid, temporary_instance_id,
         state, points_cost from adventure_sessions
         order by created_at desc limit $1",
        &[&limit.clamp(1, 200)],
    )?;
    Ok(rows.into_iter().map(from_row).collect())
}

pub fn active_session_for_player(
    client: &mut impl GenericClient,
    player_uuid: Uuid,
) -> Result<Option<AdventureSessionSummary>, StoreError> {
    let row = client.query_opt(
        "select s.id, s.adventure_kind, s.buyer_uuid, s.temporary_instance_id,
         s.state, s.points_cost from adventure_sessions s
         join adventure_participants p on p.session_id = s.id
         where p.player_uuid = $1 and s.state in ('pending','starting','ready','active')
         order by s.created_at desc limit 1",
        &[&player_uuid],
    )?;
    Ok(row.map(from_row))
}

pub fn cancel_session(
    client: &mut impl GenericClient,
    id: Uuid,
    reason: &str,
) -> Result<u64, StoreError> {
    Ok(client.execute(
        "update adventure_sessions set state = 'cancelled', failure_reason = $2,
         updated_at = now() where id = $1 and state <> 'cancelled'",
        &[&id, &reason],
    )?)
}

fn from_row(row: Row) -> AdventureSessionSummary {
    AdventureSessionSummary {
        id: row.get(0),
        adventure_kind: row.get(1),
        buyer_uuid: row.get(2),
        temporary_instance_id: row.get(3),
        state: row.get(4),
        points_cost: row.get(5),
    }
}
