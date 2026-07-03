use postgres::{GenericClient, Row};
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureSessionRecord {
    pub id: Uuid,
    pub adventure_kind: String,
    pub buyer_uuid: Uuid,
    pub temporary_instance_id: String,
    pub state: String,
    pub points_cost: i64,
}

pub struct NewAdventureSession<'a> {
    pub id: Uuid,
    pub adventure_kind: &'a str,
    pub buyer_uuid: Uuid,
    pub buyer_name: &'a str,
    pub temporary_instance_id: &'a str,
    pub points_cost: i64,
    pub points_ledger_id: Option<Uuid>,
    pub state: &'a str,
    pub start_deadline_seconds: i32,
    pub stop_deadline_seconds: i32,
    pub metadata: Value,
}

pub struct NewAdventureParticipant<'a> {
    pub session_id: Uuid,
    pub player_uuid: Uuid,
    pub player_name: &'a str,
    pub role: &'a str,
    pub state: &'a str,
    pub metadata: Value,
}

pub fn insert_session(
    client: &mut impl GenericClient,
    new: NewAdventureSession<'_>,
) -> Result<AdventureSessionRecord, StoreError> {
    let row = client.query_one(
        "insert into adventure_sessions
         (id, adventure_kind, buyer_uuid, buyer_name, temporary_instance_id,
          points_cost, points_ledger_id, state, start_deadline_at,
          stop_deadline_at, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8,
          now() + ($9::integer * interval '1 second'),
          now() + ($10::integer * interval '1 second'), $11)
         returning id, adventure_kind, buyer_uuid, temporary_instance_id,
          state, points_cost",
        &[
            &new.id,
            &new.adventure_kind,
            &new.buyer_uuid,
            &new.buyer_name,
            &new.temporary_instance_id,
            &new.points_cost,
            &new.points_ledger_id,
            &new.state,
            &new.start_deadline_seconds,
            &new.stop_deadline_seconds,
            &new.metadata,
        ],
    )?;
    Ok(session_from_row(&row))
}

pub fn update_session_state(
    client: &mut impl GenericClient,
    id: Uuid,
    state: &str,
    failure_reason: Option<&str>,
    refund_ledger_id: Option<Uuid>,
) -> Result<(), StoreError> {
    client.execute(
        "update adventure_sessions set state = $2, failure_reason = $3,
         refund_ledger_id = coalesce($4, refund_ledger_id), updated_at = now()
         where id = $1",
        &[&id, &state, &failure_reason, &refund_ledger_id],
    )?;
    Ok(())
}

pub fn add_participant(
    client: &mut impl GenericClient,
    new: NewAdventureParticipant<'_>,
) -> Result<(), StoreError> {
    client.execute(
        "insert into adventure_participants
         (session_id, player_uuid, player_name, role, state, metadata)
         values ($1, $2, $3, $4, $5, $6)",
        &[
            &new.session_id,
            &new.player_uuid,
            &new.player_name,
            &new.role,
            &new.state,
            &new.metadata,
        ],
    )?;
    Ok(())
}

pub fn get_session(
    client: &mut impl GenericClient,
    id: Uuid,
) -> Result<Option<AdventureSessionRecord>, StoreError> {
    let row = client.query_opt(
        "select id, adventure_kind, buyer_uuid, temporary_instance_id,
         state, points_cost from adventure_sessions where id = $1",
        &[&id],
    )?;
    Ok(row.map(|row| session_from_row(&row)))
}

pub fn get_session_by_instance(
    client: &mut impl GenericClient,
    instance_id: &str,
) -> Result<Option<AdventureSessionRecord>, StoreError> {
    let row = client.query_opt(
        "select id, adventure_kind, buyer_uuid, temporary_instance_id,
         state, points_cost from adventure_sessions where temporary_instance_id = $1",
        &[&instance_id],
    )?;
    Ok(row.map(|row| session_from_row(&row)))
}

pub fn mark_participant_left(
    client: &mut impl GenericClient,
    session_id: Uuid,
    player_uuid: Uuid,
) -> Result<u64, StoreError> {
    Ok(client.execute(
        "update adventure_participants set state = 'left', left_at = now()
         where session_id = $1 and player_uuid = $2",
        &[&session_id, &player_uuid],
    )?)
}

pub fn active_participant_count(
    client: &mut impl GenericClient,
    session_id: Uuid,
) -> Result<i64, StoreError> {
    let row = client.query_one(
        "select count(*)::bigint from adventure_participants
         where session_id = $1 and state not in ('left', 'failed')",
        &[&session_id],
    )?;
    Ok(row.get(0))
}

pub fn record_cleanup_event(
    client: &mut impl GenericClient,
    id: Uuid,
    instance_id: &str,
    event_kind: &str,
    result: &str,
    diagnostic: Option<&str>,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into adventure_cleanup_events
         (id, temporary_instance_id, event_kind, result, diagnostic, metadata)
         values ($1, $2, $3, $4, $5, $6)",
        &[
            &id,
            &instance_id,
            &event_kind,
            &result,
            &diagnostic,
            &metadata,
        ],
    )?;
    Ok(())
}

fn session_from_row(row: &Row) -> AdventureSessionRecord {
    AdventureSessionRecord {
        id: row.get(0),
        adventure_kind: row.get(1),
        buyer_uuid: row.get(2),
        temporary_instance_id: row.get(3),
        state: row.get(4),
        points_cost: row.get(5),
    }
}
