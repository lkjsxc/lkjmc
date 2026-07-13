use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;
use crate::data_workflows::append;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeOperation {
    pub id: Uuid,
    pub instance_id: String,
    pub correlation_id: Uuid,
    pub fence: i64,
    pub intent: String,
    pub replay: bool,
}

pub fn allocate(
    client: &mut Client,
    instance_id: &str,
    intent: &str,
    requested: &Value,
    correlation_id: Uuid,
) -> Result<RuntimeOperation, StoreError> {
    if !matches!(intent, "start" | "stop" | "observe" | "delete") || !requested.is_object() {
        return Err(StoreError::invalid_state("invalid runtime operation intent"));
    }
    let mut tx = client.transaction()?;
    if let Some(row) = tx.query_opt(
        "select operation_id, instance_id, correlation_id, fence, effect_kind
         from runtime_effect_workflows where correlation_id = $1 for update",
        &[&correlation_id],
    )? {
        let operation = from_row(&row, true);
        if operation.instance_id != instance_id || operation.intent != intent {
            return Err(StoreError::invalid_state("changed runtime operation replay"));
        }
        tx.commit()?;
        return Ok(operation);
    }
    if tx.query_opt("select 1 from instances where id = $1 for update", &[&instance_id])?.is_none() {
        return Err(StoreError::invalid_state("runtime instance missing"));
    }
    let id = Uuid::new_v4();
    let fence = tx.query_one(
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
    )?.get(0);
    tx.execute(
        "insert into runtime_effect_workflows
         (id, instance_id, effect_kind, requested_state, state, revision, fence,
          correlation_id, operation_id)
         values ($1,$2,$3,$4,'pending_observation',1,$5,$6,$1)",
        &[&id, &instance_id, &intent, requested, &fence, &correlation_id],
    )?;
    history(&mut tx, instance_id, id, correlation_id, fence, "intent", "pending", None)?;
    append(&mut tx, "runtime", id, 1, correlation_id, "pending_observation")?;
    tx.commit()?;
    Ok(RuntimeOperation {
        id, instance_id: instance_id.to_string(), correlation_id, fence,
        intent: intent.to_string(), replay: false,
    })
}

pub fn owns(client: &mut Client, operation: &RuntimeOperation) -> Result<bool, StoreError> {
    let mut tx = client.transaction()?;
    let owned = tx.query_opt(
        "select 1 from runtime_instance_fences
         where instance_id=$1 and fence=$2 and operation_id=$3 and correlation_id=$4",
        &[&operation.instance_id, &operation.fence, &operation.id, &operation.correlation_id],
    )?.is_some();
    history(
        &mut tx, &operation.instance_id, operation.id, operation.correlation_id,
        operation.fence, "ownership", if owned { "succeeded" } else { "stale" }, None,
    )?;
    tx.commit()?;
    Ok(owned)
}

pub fn observe(
    client: &mut Client,
    operation: &RuntimeOperation,
    observation: &Value,
    outcome: &str,
    detail: Option<&str>,
) -> Result<bool, StoreError> {
    if !observation.is_object() || !matches!(outcome, "succeeded" | "failed" | "unknown") {
        return Err(StoreError::invalid_state("invalid runtime observation"));
    }
    let mut tx = client.transaction()?;
    let owned = tx.query_opt(
        "select 1 from runtime_instance_fences
         where instance_id=$1 and fence=$2 and operation_id=$3 and correlation_id=$4 for update",
        &[&operation.instance_id, &operation.fence, &operation.id, &operation.correlation_id],
    )?.is_some();
    if !owned {
        history(&mut tx, &operation.instance_id, operation.id, operation.correlation_id,
            operation.fence, "observation", "stale", detail)?;
        tx.commit()?;
        return Ok(false);
    }
    let state = if outcome == "succeeded" { "observed" } else if outcome == "failed" { "failed" } else { "pending_observation" };
    let revision: i64 = tx.query_one(
        "update runtime_effect_workflows set state=$2, revision=revision+1,
         observation=$3, failure_reason=$4, updated_at=now()
         where operation_id=$1 returning revision",
        &[&operation.id, &state, observation, &detail],
    )?.get(0);
    history(&mut tx, &operation.instance_id, operation.id, operation.correlation_id,
        operation.fence, "outcome", outcome, detail)?;
    append(&mut tx, "runtime", operation.id, revision, operation.correlation_id, state)?;
    tx.commit()?;
    Ok(true)
}

pub fn history_count(client: &mut Client, operation_id: Uuid) -> Result<i64, StoreError> {
    Ok(client.query_one(
        "select count(*) from runtime_reconcile_history where operation_id=$1",
        &[&operation_id],
    )?.get(0))
}

fn history(
    client: &mut impl postgres::GenericClient,
    instance_id: &str, operation_id: Uuid, correlation_id: Uuid, fence: i64,
    phase: &str, outcome: &str, detail: Option<&str>,
) -> Result<(), StoreError> {
    client.execute(
        "insert into runtime_reconcile_history
         (instance_id,operation_id,correlation_id,fence,attempt,phase,outcome,detail)
         values ($1,$2,$3,$4,
           (select coalesce(max(attempt),0)+1 from runtime_reconcile_history where operation_id=$2),
           $5,$6,$7)",
        &[&instance_id, &operation_id, &correlation_id, &fence, &phase, &outcome, &detail],
    )?;
    Ok(())
}

fn from_row(row: &postgres::Row, replay: bool) -> RuntimeOperation {
    RuntimeOperation {
        id: row.get(0), instance_id: row.get(1), correlation_id: row.get(2),
        fence: row.get(3), intent: row.get(4), replay,
    }
}
