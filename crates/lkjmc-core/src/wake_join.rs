use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WakeJoinState {
    Queued,
    Starting,
    Ready,
    Transferred,
    Failed,
    Cancelled,
    Expired,
    Denied,
}

impl WakeJoinState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Transferred => "transferred",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
            Self::Denied => "denied",
        }
    }

    pub fn final_state(self) -> bool {
        matches!(
            self,
            Self::Transferred | Self::Failed | Self::Cancelled | Self::Expired | Self::Denied
        )
    }
}

pub fn can_cancel(state: WakeJoinState) -> bool {
    matches!(
        state,
        WakeJoinState::Queued | WakeJoinState::Starting | WakeJoinState::Ready
    )
}

pub fn can_consume(state: WakeJoinState) -> bool {
    matches!(state, WakeJoinState::Ready)
}

pub fn cleanup_state(state: WakeJoinState, expired: bool) -> WakeJoinState {
    if expired && !state.final_state() {
        WakeJoinState::Expired
    } else {
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_states_are_total() {
        assert!(WakeJoinState::Transferred.final_state());
        assert!(!WakeJoinState::Ready.final_state());
        assert!(can_cancel(WakeJoinState::Starting));
        assert!(can_consume(WakeJoinState::Ready));
        assert_eq!(
            cleanup_state(WakeJoinState::Queued, true),
            WakeJoinState::Expired
        );
    }
}
