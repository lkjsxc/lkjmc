use serde::{Deserialize, Serialize};

use crate::instance::{DesiredState, InstanceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutosuspendPolicy {
    pub enabled: bool,
    pub idle_grace_seconds: u64,
    pub minimum_uptime_seconds: u64,
    pub heartbeat_stale_seconds: u64,
    pub empty_heartbeat_count: u32,
    pub stop_when_empty: bool,
    pub delete_when_expired: bool,
    pub keep_warm: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutosuspendInput {
    pub kind: InstanceKind,
    pub desired_state: DesiredState,
    pub observed_running: bool,
    pub heartbeat_age_seconds: Option<u64>,
    pub player_count: Option<u32>,
    pub active_sessions: u32,
    pub uptime_seconds: Option<u64>,
    pub empty_since_age_seconds: Option<u64>,
    pub consecutive_empty_heartbeats: u32,
    pub policy: AutosuspendPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutosuspendDecision {
    Noop,
    SetEmptySince,
    ClearEmptySince,
    MarkSuspendedAndStop { reason: String },
    Skip { reason: String },
}

impl Default for AutosuspendPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_grace_seconds: 300,
            minimum_uptime_seconds: 120,
            heartbeat_stale_seconds: 90,
            empty_heartbeat_count: 2,
            stop_when_empty: true,
            delete_when_expired: false,
            keep_warm: false,
        }
    }
}

pub fn plan(input: AutosuspendInput) -> AutosuspendDecision {
    if !input.policy.enabled {
        return skip("autosuspend disabled");
    }
    if input.policy.keep_warm || input.kind == InstanceKind::Velocity {
        return skip("instance is keep-warm");
    }
    if input.desired_state == DesiredState::Suspended {
        return AutosuspendDecision::Noop;
    }
    if input.desired_state != DesiredState::Running && input.desired_state != DesiredState::Starting
    {
        return skip("desired state is not running");
    }
    if !input.observed_running {
        return skip("runtime is not running");
    }
    let Some(age) = input.heartbeat_age_seconds else {
        return skip("presence is unknown");
    };
    if age > input.policy.heartbeat_stale_seconds {
        return skip("heartbeat is stale");
    }
    let Some(players) = input.player_count else {
        return skip("player count is unknown");
    };
    if input.active_sessions > 0 {
        return skip("active sessions exist");
    }
    if players > 0 {
        return if input.empty_since_age_seconds.is_some() {
            AutosuspendDecision::ClearEmptySince
        } else {
            AutosuspendDecision::Noop
        };
    }
    if input
        .uptime_seconds
        .is_some_and(|uptime| uptime < input.policy.minimum_uptime_seconds)
    {
        return skip("minimum uptime not reached");
    }
    if input.empty_since_age_seconds.is_none() {
        return AutosuspendDecision::SetEmptySince;
    }
    if input.consecutive_empty_heartbeats < input.policy.empty_heartbeat_count {
        return AutosuspendDecision::Noop;
    }
    if input.empty_since_age_seconds.unwrap_or_default() < input.policy.idle_grace_seconds {
        return AutosuspendDecision::Noop;
    }
    if !input.policy.stop_when_empty {
        return skip("stopWhenEmpty is false");
    }
    AutosuspendDecision::MarkSuspendedAndStop {
        reason: "empty after idle grace".to_string(),
    }
}

pub fn manual_start_state() -> DesiredState {
    DesiredState::Running
}

fn skip(reason: &'static str) -> AutosuspendDecision {
    AutosuspendDecision::Skip {
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod autosuspend_tests;
