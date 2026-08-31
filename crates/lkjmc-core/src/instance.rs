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

impl DesiredState {
    pub const ALL: &'static [(Self, &'static str)] = &[
        (Self::Stopped, "stopped"),
        (Self::Starting, "starting"),
        (Self::Running, "running"),
        (Self::Suspended, "suspended"),
        (Self::Stopping, "stopping"),
        (Self::Restarting, "restarting"),
        (Self::Deleting, "deleting"),
        (Self::Failed, "failed"),
    ];

    pub fn requires_service(self) -> bool {
        matches!(self, Self::Starting | Self::Running | Self::Restarting)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Suspended => "suspended",
            Self::Stopping => "stopping",
            Self::Restarting => "restarting",
            Self::Deleting => "deleting",
            Self::Failed => "failed",
        }
    }
}

impl InstanceKind {
    pub const ALL: &'static [(Self, &'static str)] = &[
        (Self::Velocity, "velocity"),
        (Self::Paper, "paper"),
        (Self::Folia, "folia"),
        (Self::Purpur, "purpur"),
        (Self::VanillaCustom, "vanilla-custom"),
        (Self::ModdedCustom, "modded-custom"),
    ];

    pub fn requires_minecraft_eula(self) -> bool {
        !matches!(self, Self::Velocity)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Velocity => "velocity",
            Self::Paper => "paper",
            Self::Folia => "folia",
            Self::Purpur => "purpur",
            Self::VanillaCustom => "vanilla-custom",
            Self::ModdedCustom => "modded-custom",
        }
    }
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
