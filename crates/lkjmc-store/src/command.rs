use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

pub fn insert_requested(
    client: &mut Client,
    id: Uuid,
    actor_kind: &str,
    actor_name: &str,
    command: &str,
    body: &Value,
) -> Result<(), StoreError> {
    let metadata = Value::Object(Default::default());
    client.execute(
        "insert into commands
         (id, actor_kind, actor_name, command, body, result, metadata)
         values ($1, $2, $3, $4, $5, 'requested', $6)",
        &[&id, &actor_kind, &actor_name, &command, &body, &metadata],
    )?;
    Ok(())
}
