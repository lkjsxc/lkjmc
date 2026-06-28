use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemporaryLifecycleState {
    Planned,
    Created,
    Starting,
    Ready,
    Stopping,
    Stopped,
    Failed,
    Cleaning,
    Cleaned,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CleanupPolicy {
    Delete,
    Archive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdventureSessionState {
    Pending,
    Starting,
    Ready,
    Active,
    Completed,
    Failed,
    Refunded,
    Cancelled,
    Expired,
}

impl TemporaryLifecycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Created => "created",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
            Self::Cleaning => "cleaning",
            Self::Cleaned => "cleaned",
            Self::Archived => "archived",
        }
    }
}

impl CleanupPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delete => "delete",
            Self::Archive => "archive",
        }
    }
}

impl AdventureSessionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Refunded => "refunded",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        }
    }
}

pub fn can_start(state: TemporaryLifecycleState) -> bool {
    matches!(
        state,
        TemporaryLifecycleState::Created | TemporaryLifecycleState::Stopped
    )
}

pub fn needs_refund_after_start_failure(state: AdventureSessionState) -> bool {
    matches!(
        state,
        AdventureSessionState::Pending | AdventureSessionState::Starting
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_refund_rules_are_explicit() {
        assert!(can_start(TemporaryLifecycleState::Created));
        assert!(!can_start(TemporaryLifecycleState::Ready));
        assert!(needs_refund_after_start_failure(
            AdventureSessionState::Starting
        ));
        assert!(!needs_refund_after_start_failure(
            AdventureSessionState::Refunded
        ));
    }
}
