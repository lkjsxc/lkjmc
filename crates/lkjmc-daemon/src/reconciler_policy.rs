use lkjmc_core::autosuspend::AutosuspendPolicy;
use lkjmc_core::instance::{DesiredState, InstanceKind};
use serde_json::Value;

pub(crate) fn policy(kind: InstanceKind, id: &str, value: Option<&Value>) -> AutosuspendPolicy {
    let mut policy = AutosuspendPolicy {
        enabled: kind != InstanceKind::Velocity,
        keep_warm: kind == InstanceKind::Velocity || id == "hub",
        ..AutosuspendPolicy::default()
    };
    let Some(value) = value else {
        return policy;
    };
    if let Some(enabled) = value.get("enabled").and_then(Value::as_bool) {
        policy.enabled = enabled;
    }
    if let Some(keep) = value.get("keepWarm").and_then(Value::as_bool) {
        policy.keep_warm = keep;
    }
    if let Some(seconds) = value.get("idleGraceSeconds").and_then(Value::as_u64) {
        policy.idle_grace_seconds = seconds;
    }
    if let Some(seconds) = value.get("minimumUptimeSeconds").and_then(Value::as_u64) {
        policy.minimum_uptime_seconds = seconds;
    }
    if let Some(seconds) = value.get("heartbeatStaleSeconds").and_then(Value::as_u64) {
        policy.heartbeat_stale_seconds = seconds;
    }
    if let Some(count) = value
        .get("emptyHeartbeatCount")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
    {
        policy.empty_heartbeat_count = count;
    }
    if let Some(stop) = value.get("stopWhenEmpty").and_then(Value::as_bool) {
        policy.stop_when_empty = stop;
    }
    policy
}

pub(crate) fn kind(value: &str) -> Option<InstanceKind> {
    match value {
        "velocity" => Some(InstanceKind::Velocity),
        "paper" => Some(InstanceKind::Paper),
        "folia" => Some(InstanceKind::Folia),
        "purpur" => Some(InstanceKind::Purpur),
        "vanilla-custom" => Some(InstanceKind::VanillaCustom),
        "modded-custom" => Some(InstanceKind::ModdedCustom),
        _ => None,
    }
}

pub(crate) fn desired(value: &str) -> Option<DesiredState> {
    match value {
        "stopped" => Some(DesiredState::Stopped),
        "starting" => Some(DesiredState::Starting),
        "running" => Some(DesiredState::Running),
        "suspended" => Some(DesiredState::Suspended),
        "stopping" => Some(DesiredState::Stopping),
        "restarting" => Some(DesiredState::Restarting),
        "deleting" => Some(DesiredState::Deleting),
        "failed" => Some(DesiredState::Failed),
        _ => None,
    }
}
