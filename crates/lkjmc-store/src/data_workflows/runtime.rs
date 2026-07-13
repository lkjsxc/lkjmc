use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::error::StoreError;

use super::{change_feed, record, WorkflowRecord};

pub struct NewRuntimeIntent<'a> {
    pub id: Uuid,
    pub instance_id: &'a str,
    pub effect_kind: &'a str,
    pub requested_state: Value,
    pub fence: i64,
    pub correlation_id: Uuid,
}

pub fn create_runtime_intent(
    client: &mut Client,
    new: NewRuntimeIntent<'_>,
) -> Result<WorkflowRecord, StoreError> {
    if new.effect_kind.is_empty() || !new.requested_state.is_object() || new.fence < 1 {
        return Err(StoreError::invalid_state("invalid runtime effect intent"));
    }
    let mut tx = client.transaction()?;
    if let Some(row) = tx.query_opt(
        "select id, state, revision, fence, correlation_id, instance_id,
         effect_kind, requested_state from runtime_effect_workflows
         where correlation_id = $1 for update",
        &[&new.correlation_id],
    )? {
        let same = row.get::<_, Uuid>(0) == new.id
            && row.get::<_, String>(5) == new.instance_id
            && row.get::<_, String>(6) == new.effect_kind
            && row.get::<_, Value>(7) == new.requested_state
            && row.get::<_, i64>(3) == new.fence;
        if !same {
            return Err(StoreError::invalid_state("changed runtime intent replay"));
        }
        let result = record(&row, true)?;
        tx.commit()?;
        return Ok(result);
    }
    if tx
        .query_opt(
            "select 1 from instances where id = $1 for update",
            &[&new.instance_id],
        )?
        .is_none()
    {
        return Err(StoreError::invalid_state("runtime instance missing"));
    }
    let row = tx.query_one(
        "insert into runtime_effect_workflows
         (id, instance_id, effect_kind, requested_state, state, revision, fence, correlation_id,
          operation_id)
         values ($1,$2,$3,$4,'pending_observation',1,$5,$6,$1)
         returning id, state, revision, fence, correlation_id",
        &[
            &new.id,
            &new.instance_id,
            &new.effect_kind,
            &new.requested_state,
            &new.fence,
            &new.correlation_id,
        ],
    )?;
    change_feed::append(
        &mut tx,
        "runtime",
        new.id,
        1,
        new.correlation_id,
        "pending_observation",
    )?;
    let result = record(&row, false)?;
    tx.commit()?;
    Ok(result)
}
