use serde::{Deserialize, Serialize};

use crate::instance::{DesiredState, ObservedState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconcileInput {
    pub desired: DesiredState,
    pub observed: ObservedState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReconcileEffect {
    StartInstance,
    StopInstance,
    DeleteInstanceFiles,
    RecordStable,
    RecordFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconcilePlan {
    pub effects: Vec<ReconcileEffect>,
}

pub fn plan(input: ReconcileInput) -> ReconcilePlan {
    let effect = match (input.desired, input.observed) {
        (DesiredState::Running | DesiredState::Starting, ObservedState::ProcessAbsent)
        | (DesiredState::Running | DesiredState::Starting, ObservedState::ProcessExited)
        | (DesiredState::Running | DesiredState::Starting, ObservedState::ProcessUnknown) => {
            ReconcileEffect::StartInstance
        }
        (DesiredState::Stopped | DesiredState::Stopping, ObservedState::ProcessHealthy)
        | (DesiredState::Stopped | DesiredState::Stopping, ObservedState::ProcessStarting)
        | (DesiredState::Stopped | DesiredState::Stopping, ObservedState::ProcessUnhealthy) => {
            ReconcileEffect::StopInstance
        }
        (DesiredState::Restarting, ObservedState::ProcessAbsent)
        | (DesiredState::Restarting, ObservedState::ProcessExited) => {
            ReconcileEffect::StartInstance
        }
        (DesiredState::Restarting, _) => ReconcileEffect::StopInstance,
        (DesiredState::Deleting, ObservedState::ProcessAbsent)
        | (DesiredState::Deleting, ObservedState::ProcessExited) => {
            ReconcileEffect::DeleteInstanceFiles
        }
        (DesiredState::Deleting, _) => ReconcileEffect::StopInstance,
        (DesiredState::Failed, _) => ReconcileEffect::RecordFailed,
        _ => ReconcileEffect::RecordStable,
    };
    ReconcilePlan {
        effects: vec![effect],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_missing_running_instance() {
        let plan = plan(ReconcileInput {
            desired: DesiredState::Running,
            observed: ObservedState::ProcessAbsent,
        });
        assert_eq!(plan.effects, vec![ReconcileEffect::StartInstance]);
    }

    #[test]
    fn stops_running_process_when_desired_stopped() {
        let plan = plan(ReconcileInput {
            desired: DesiredState::Stopped,
            observed: ObservedState::ProcessHealthy,
        });
        assert_eq!(plan.effects, vec![ReconcileEffect::StopInstance]);
    }
}
