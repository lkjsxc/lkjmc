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
