use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowKind {
    Transfer,
    Delivery,
    Adventure,
    Runtime,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    PendingSave,
    SaveAcknowledged,
    PendingArrival,
    Arrived,
    PendingReceipt,
    Received,
    PendingStart,
    StartObserved,
    PendingCleanup,
    Cleaned,
    PendingObservation,
    Observed,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransitionPlan {
    pub from: WorkflowState,
    pub to: WorkflowState,
    pub next_revision: i64,
}

pub fn initial_state(kind: WorkflowKind) -> WorkflowState {
    match kind {
        WorkflowKind::Transfer => WorkflowState::PendingSave,
        WorkflowKind::Delivery => WorkflowState::PendingReceipt,
        WorkflowKind::Adventure => WorkflowState::PendingStart,
        WorkflowKind::Runtime => WorkflowState::PendingObservation,
    }
}

pub fn plan_failure(state: WorkflowState, revision: i64) -> Result<TransitionPlan, String> {
    if revision < 1 {
        return Err("workflow revision must be positive".into());
    }
    if terminal(state) {
        return Err("terminal workflow cannot transition".into());
    }
    let next_revision = revision
        .checked_add(1)
        .ok_or_else(|| "workflow revision exhausted".to_string())?;
    Ok(TransitionPlan {
        from: state,
        to: WorkflowState::Failed,
        next_revision,
    })
}

pub fn terminal(state: WorkflowState) -> bool {
    matches!(
        state,
        WorkflowState::Arrived
            | WorkflowState::Received
            | WorkflowState::Cleaned
            | WorkflowState::Observed
            | WorkflowState::Failed
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_owner_can_only_plan_failure() -> Result<(), String> {
        for kind in [
            WorkflowKind::Transfer,
            WorkflowKind::Delivery,
            WorkflowKind::Adventure,
            WorkflowKind::Runtime,
        ] {
            let initial = initial_state(kind);
            let plan = plan_failure(initial, 3)?;
            assert_eq!(plan.to, WorkflowState::Failed);
            assert_eq!(plan.next_revision, 4);
            assert!(plan_failure(WorkflowState::Failed, 4).is_err());
        }
        Ok(())
    }
}
