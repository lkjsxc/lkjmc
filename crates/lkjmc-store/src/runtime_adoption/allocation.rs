use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use super::{history, RuntimeOperation};
use crate::data_workflows::append;
use crate::error::StoreError;

pub fn allocate(
    client: &mut Client,
    instance_id: &str,
    intent: &str,
    requested: &Value,
    correlation_id: Uuid,
) -> Result<RuntimeOperation, StoreError> {
    if !matches!(intent, "start" | "stop" | "observe" | "delete") || !requested.is_object() {
        return Err(StoreError::invalid_state(
            "invalid runtime operation intent",
        ));
    }
    let mut tx = client.transaction()?;
    if let Some(row) = tx.query_opt(
        "select operation_id, instance_id, correlation_id, fence, effect_kind
         from runtime_effect_workflows where correlation_id = $1 for update",
        &[&correlation_id],
    )? {
        let operation = history::from_row(&row, true);
        if operation.instance_id != instance_id || operation.intent != intent {
            return Err(StoreError::invalid_state(
                "changed runtime operation replay",
            ));
        }
        tx.commit()?;
        return Ok(operation);
    }
    if tx
        .query_opt(
            "select 1 from instances where id = $1 for update",
            &[&instance_id],
        )?
        .is_none()
    {
        return Err(StoreError::invalid_state("runtime instance missing"));
    }
    let id = Uuid::new_v4();
    let fence = tx
        .query_one(
            "insert into runtime_instance_fences
         (instance_id, fence, operation_id, correlation_id, intent)
         values ($1,1,$2,$3,$4)
         on conflict (instance_id) do update set
           fence = runtime_instance_fences.fence + 1,
           operation_id = excluded.operation_id,
           correlation_id = excluded.correlation_id,
           intent = excluded.intent,
           updated_at = now()
         returning fence",
            &[&instance_id, &id, &correlation_id, &intent],
        )?
        .get(0);
    tx.execute(
        "insert into runtime_effect_workflows
         (id, instance_id, effect_kind, requested_state, state, revision, fence,
          correlation_id, operation_id)
         values ($1,$2,$3,$4,'pending_observation',1,$5,$6,$1)",
        &[
            &id,
            &instance_id,
            &intent,
            requested,
            &fence,
            &correlation_id,
        ],
    )?;
    let operation = RuntimeOperation {
        id,
        instance_id: instance_id.to_string(),
        correlation_id,
        fence,
        intent: intent.to_string(),
        replay: false,
    };
    history::append(&mut tx, &operation, "intent", "pending", None)?;
    append(
        &mut tx,
        "runtime",
        id,
        1,
        correlation_id,
        "pending_observation",
    )?;
    tx.commit()?;
    Ok(operation)
}
