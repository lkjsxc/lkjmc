use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

pub fn insert(
    client: &mut Client,
    id: Uuid,
    topic: &str,
    payload: &Value,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into outbox_events (id, topic, payload, metadata) values ($1, $2, $3, $4)",
        &[&id, &topic, &payload, &metadata],
    )?;
    Ok(())
}
