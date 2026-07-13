mod change_feed;
mod delivery;
mod runtime;
mod transfer;

pub(crate) use change_feed::append;
pub use change_feed::{
    changes_after, retained_floor, run_retention, ChangeRecord, ResumeResult, RetentionResult,
};
pub(crate) use delivery::insert_delivery;
pub use delivery::{create_delivery, NewDelivery};
pub use runtime::{create_runtime_intent, NewRuntimeIntent};
pub use transfer::{create_transfer, NewTransfer};

use postgres::{Client, GenericClient, Row};
use uuid::Uuid;

use lkjmc_core::data_workflow::{plan_failure, WorkflowState};

use crate::error::StoreError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRecord {
    pub id: Uuid,
    pub state: WorkflowState,
    pub revision: i64,
    pub fence: i64,
    pub correlation_id: Uuid,
    pub replay: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowTable {
    Transfer,
    Delivery,
    Adventure,
    Runtime,
}

impl WorkflowTable {
    fn sql_name(self) -> &'static str {
        match self {
            Self::Transfer => "transfer_workflows",
            Self::Delivery => "item_delivery_workflows",
            Self::Adventure => "adventure_sessions",
            Self::Runtime => "runtime_effect_workflows",
        }
    }
    fn aggregate(self) -> &'static str {
        match self {
            Self::Transfer => "transfer",
            Self::Delivery => "delivery",
            Self::Adventure => "adventure",
            Self::Runtime => "runtime",
        }
    }
}

pub fn fail(
    client: &mut Client,
    table: WorkflowTable,
    id: Uuid,
    expected_revision: i64,
    fence: i64,
    reason: &str,
) -> Result<WorkflowRecord, StoreError> {
    let mut tx = client.transaction()?;
    let result = fail_in_transaction(&mut tx, table, id, expected_revision, fence, reason)?;
    tx.commit()?;
    Ok(result)
}

pub(crate) fn fail_in_transaction(
    client: &mut impl GenericClient,
    table: WorkflowTable,
    id: Uuid,
    expected_revision: i64,
    fence: i64,
    reason: &str,
) -> Result<WorkflowRecord, StoreError> {
    if reason.is_empty() || reason.len() > 1024 {
        return Err(StoreError::invalid_state("invalid workflow failure reason"));
    }
    let select = format!(
        "select id, state, revision, fence, correlation_id, failure_reason
         from {} where id = $1 for update",
        table.sql_name()
    );
    let row = client
        .query_opt(&select, &[&id])?
        .ok_or_else(|| StoreError::invalid_state("workflow missing"))?;
    let current = record(&row, false)?;
    if current.state == WorkflowState::Failed {
        let next_revision = expected_revision
            .checked_add(1)
            .ok_or_else(|| StoreError::invalid_state("workflow revision exhausted"))?;
        let stored_reason: Option<String> = row.get(5);
        if current.revision == next_revision
            && current.fence == fence
            && stored_reason.as_deref() == Some(reason)
        {
            return Ok(WorkflowRecord {
                replay: true,
                ..current
            });
        }
        return Err(StoreError::invalid_state("changed workflow replay"));
    }
    if current.revision != expected_revision || current.fence != fence {
        return Err(StoreError::invalid_state(
            "stale workflow revision or fence",
        ));
    }
    let plan = plan_failure(current.state, current.revision).map_err(StoreError::invalid_state)?;
    let update = format!("update {} set state = 'failed', revision = $2, failure_reason = $3, updated_at = now() where id = $1", table.sql_name());
    client.execute(&update, &[&id, &plan.next_revision, &reason])?;
    change_feed::append(
        client,
        table.aggregate(),
        id,
        plan.next_revision,
        current.correlation_id,
        "failed",
    )?;
    Ok(WorkflowRecord {
        state: WorkflowState::Failed,
        revision: plan.next_revision,
        ..current
    })
}

pub(crate) fn record(row: &Row, replay: bool) -> Result<WorkflowRecord, StoreError> {
    let state: String = row.get(1);
    let state = serde_json::from_value(serde_json::Value::String(state))
        .map_err(|_| StoreError::invalid_state("unknown workflow state"))?;
    Ok(WorkflowRecord {
        id: row.get(0),
        state,
        revision: row.get(2),
        fence: row.get(3),
        correlation_id: row.get(4),
        replay,
    })
}
