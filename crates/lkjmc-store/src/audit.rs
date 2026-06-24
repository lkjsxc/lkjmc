use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

pub struct NewAuditEvent<'a> {
    pub id: Uuid,
    pub actor_kind: &'a str,
    pub actor_name: &'a str,
    pub action: &'a str,
    pub target_kind: &'a str,
    pub target_id: &'a str,
    pub result: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditTailRow {
    pub actor_kind: String,
    pub actor_name: String,
    pub action: String,
    pub target_kind: String,
    pub target_id: String,
    pub result: String,
}

pub fn insert(client: &mut Client, event: NewAuditEvent<'_>) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into audit_events
         (id, actor_kind, actor_name, action, target_kind, target_id, result, metadata)
         values ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[
            &event.id,
            &event.actor_kind,
            &event.actor_name,
            &event.action,
            &event.target_kind,
            &event.target_id,
            &event.result,
            &metadata,
        ],
    )?;
    Ok(())
}

pub fn count(client: &mut Client) -> Result<i64, StoreError> {
    let row = client.query_one("select count(*)::bigint from audit_events", &[])?;
    Ok(row.get(0))
}

pub fn tail(client: &mut Client, limit: i64) -> Result<Vec<AuditTailRow>, StoreError> {
    let rows = client.query(
        "select actor_kind, actor_name, action, target_kind, target_id, result
         from audit_events order by created_at desc limit $1",
        &[&limit],
    )?;
    Ok(rows
        .into_iter()
        .map(|row| AuditTailRow {
            actor_kind: row.get(0),
            actor_name: row.get(1),
            action: row.get(2),
            target_kind: row.get(3),
            target_id: row.get(4),
            result: row.get(5),
        })
        .collect())
}
