use serde::{Deserialize, Serialize};

use super::desired::DesiredNetwork;
use super::diagnostic::{BootstrapDiagnostic, DiagnosticSeverity};
use super::effect::BootstrapEffect;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapPlan {
    pub desired_network: DesiredNetwork,
    pub effects: Vec<BootstrapEffect>,
    pub rollback: Vec<BootstrapEffect>,
    pub diagnostics: Vec<BootstrapDiagnostic>,
    pub outcome: PlannedOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlannedOutcome {
    Blocked,
    ReadyToApply,
    AlreadyConverged,
    DryRun,
}

impl BootstrapPlan {
    pub fn new(
        desired_network: DesiredNetwork,
        effects: Vec<BootstrapEffect>,
        diagnostics: Vec<BootstrapDiagnostic>,
        dry_run: bool,
    ) -> Self {
        let blocked = diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Blocking);
        let outcome = if blocked {
            PlannedOutcome::Blocked
        } else if effects.is_empty() {
            PlannedOutcome::AlreadyConverged
        } else if dry_run {
            PlannedOutcome::DryRun
        } else {
            PlannedOutcome::ReadyToApply
        };
        Self {
            desired_network,
            effects,
            rollback: Vec::new(),
            diagnostics,
            outcome,
        }
    }
}
