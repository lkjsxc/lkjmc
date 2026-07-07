use serde::{Deserialize, Serialize};

use crate::id::InstanceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstanceKind {
    Velocity,
    Paper,
    Folia,
    Purpur,
    VanillaCustom,
    ModdedCustom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesiredState {
    Stopped,
    Starting,
    Running,
    Suspended,
    Stopping,
    Restarting,
    Deleting,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservedState {
    ProcessAbsent,
    ProcessStarting,
    ProcessHealthy,
    ProcessUnhealthy,
    ProcessExited,
    ProcessUnknown,
    KubernetesAbsent,
    KubernetesStarting,
    KubernetesReady,
    KubernetesUnhealthy,
    KubernetesExited,
    KubernetesUnknown,
    RuntimeAbsent,
    RuntimeStarting,
    RuntimeReady,
    RuntimeUnhealthy,
    RuntimeExited,
    RuntimeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceDesired {
    pub id: InstanceId,
    pub kind: InstanceKind,
    pub desired_state: DesiredState,
    pub jar_ref: String,
    pub memory_mb: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceObservation {
    pub id: InstanceId,
    pub observed_state: ObservedState,
    pub pid: Option<u32>,
    pub healthy: bool,
    pub message: Option<String>,
}
