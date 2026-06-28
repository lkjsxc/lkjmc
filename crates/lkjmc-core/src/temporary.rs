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

pub struct TemporaryInstanceRequest<'a> {
    pub instance_id: &'a str,
    pub world_root: &'a str,
    pub max_lifetime_seconds: u32,
    pub retention_seconds: u32,
    pub cleanup_policy: CleanupPolicy,
}

pub struct TemporaryRuntimeFacts {
    pub occupied_ports: Vec<u16>,
    pub port_range_start: u16,
    pub port_range_end: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryInstancePlan {
    pub instance_id: String,
    pub server_port: u16,
    pub world_path: String,
    pub visibility: String,
    pub max_lifetime_seconds: u32,
    pub retention_seconds: u32,
    pub cleanup_policy: CleanupPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemporaryPlanError {
    EmptyInstanceId,
    EmptyWorldRoot,
    InvalidLifetime,
    NoPortAvailable,
}

pub fn plan_temporary_instance(
    request: &TemporaryInstanceRequest<'_>,
    facts: &TemporaryRuntimeFacts,
) -> Result<TemporaryInstancePlan, TemporaryPlanError> {
    if request.instance_id.is_empty() {
        return Err(TemporaryPlanError::EmptyInstanceId);
    }
    if request.world_root.is_empty() {
        return Err(TemporaryPlanError::EmptyWorldRoot);
    }
    if request.max_lifetime_seconds == 0 {
        return Err(TemporaryPlanError::InvalidLifetime);
    }
    let Some(server_port) = (facts.port_range_start..=facts.port_range_end)
        .find(|port| !facts.occupied_ports.contains(port))
    else {
        return Err(TemporaryPlanError::NoPortAvailable);
    };
    let world_root = request.world_root.trim_end_matches('/');
    Ok(TemporaryInstancePlan {
        instance_id: request.instance_id.to_string(),
        server_port,
        world_path: format!("{world_root}/{}", request.instance_id),
        visibility: "hidden".to_string(),
        max_lifetime_seconds: request.max_lifetime_seconds,
        retention_seconds: request.retention_seconds,
        cleanup_policy: request.cleanup_policy,
    })
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

    #[test]
    fn temporary_plan_allocates_hidden_world_and_port() {
        let planned = plan_temporary_instance(
            &TemporaryInstanceRequest {
                instance_id: "temp-end-1",
                world_root: "/srv/lkjmc/worlds/",
                max_lifetime_seconds: 3600,
                retention_seconds: 600,
                cleanup_policy: CleanupPolicy::Delete,
            },
            &TemporaryRuntimeFacts {
                occupied_ports: vec![30000],
                port_range_start: 30000,
                port_range_end: 30001,
            },
        );
        assert!(planned.is_ok(), "{planned:?}");
        let Ok(plan) = planned else {
            return;
        };
        assert_eq!(plan.server_port, 30001);
        assert_eq!(plan.world_path, "/srv/lkjmc/worlds/temp-end-1");
        assert_eq!(plan.visibility, "hidden");
    }
}
