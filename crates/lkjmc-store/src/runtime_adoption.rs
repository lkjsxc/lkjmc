use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

mod allocation;
mod history;
mod observation;

pub use allocation::allocate;
pub use observation::observe;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeOperation {
    pub id: Uuid,
    pub instance_id: String,
    pub correlation_id: Uuid,
    pub fence: i64,
    pub intent: String,
    pub replay: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingRuntimeOperation {
    pub operation: RuntimeOperation,
    pub observation: Option<Value>,
    pub effect_started: bool,
}

pub fn pending(
    client: &mut Client,
    instance_id: &str,
) -> Result<Option<PendingRuntimeOperation>, StoreError> {
    let row = client.query_opt(
        "select w.operation_id, w.instance_id, w.correlation_id, w.fence,
                w.effect_kind, w.observation,
                exists(select 1 from runtime_reconcile_history h
                       where h.operation_id=w.operation_id and h.phase='effect'
                         and h.outcome='pending')
         from runtime_instance_fences f
         join runtime_effect_workflows w on w.operation_id = f.operation_id
         where f.instance_id=$1 and w.state='pending_observation'",
        &[&instance_id],
    )?;
    Ok(row.map(|row| PendingRuntimeOperation {
        operation: history::from_row(&row, true),
        observation: row.get(5),
        effect_started: row.get(6),
    }))
}

pub fn is_pending(client: &mut Client, operation: &RuntimeOperation) -> Result<bool, StoreError> {
    Ok(client
        .query_opt(
            "select 1 from runtime_effect_workflows
         where operation_id=$1 and state='pending_observation'",
            &[&operation.id],
        )?
        .is_some())
}

pub fn latest_observation(
    client: &mut Client,
    instance_id: &str,
) -> Result<Option<Value>, StoreError> {
    Ok(client
        .query_opt(
            "select observation from runtime_effect_workflows
         where instance_id=$1 and observation is not null order by updated_at desc limit 1",
            &[&instance_id],
        )?
        .and_then(|row| row.get(0)))
}

pub fn owns(client: &mut Client, operation: &RuntimeOperation) -> Result<bool, StoreError> {
    let mut tx = client.transaction()?;
    let owned = tx
        .query_opt(
            "select 1 from runtime_instance_fences
         where instance_id=$1 and fence=$2 and operation_id=$3 and correlation_id=$4",
            &[
                &operation.instance_id,
                &operation.fence,
                &operation.id,
                &operation.correlation_id,
            ],
        )?
        .is_some();
    history::append(
        &mut tx,
        operation,
        "ownership",
        if owned { "succeeded" } else { "stale" },
        None,
    )?;
    tx.commit()?;
    Ok(owned)
}

pub fn mark_effect(client: &mut Client, operation: &RuntimeOperation) -> Result<bool, StoreError> {
    let mut tx = client.transaction()?;
    let owned = tx
        .query_opt(
            "select 1 from runtime_instance_fences
         where instance_id=$1 and fence=$2 and operation_id=$3 and correlation_id=$4 for update",
            &[
                &operation.instance_id,
                &operation.fence,
                &operation.id,
                &operation.correlation_id,
            ],
        )?
        .is_some();
    history::append(
        &mut tx,
        operation,
        "effect",
        if owned { "pending" } else { "stale" },
        None,
    )?;
    tx.commit()?;
    Ok(owned)
}

pub fn history_count(client: &mut Client, operation_id: Uuid) -> Result<i64, StoreError> {
    history::count(client, operation_id)
}
