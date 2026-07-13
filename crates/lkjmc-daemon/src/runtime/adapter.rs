use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeCapabilities {
    pub process_identity: bool,
    pub readiness: bool,
    pub storage: bool,
    pub secrets: bool,
    pub configuration: bool,
    pub logs: bool,
    pub recovery: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RuntimeRequirements {
    pub readiness: bool,
    pub storage: bool,
    pub secrets: bool,
    pub configuration: bool,
    pub logs: bool,
    pub recovery: bool,
}

pub fn require(
    capabilities: RuntimeCapabilities,
    requirements: RuntimeRequirements,
) -> Result<(), String> {
    let checks = [
        (requirements.readiness, capabilities.readiness, "readiness"),
        (requirements.storage, capabilities.storage, "storage"),
        (requirements.secrets, capabilities.secrets, "secrets"),
        (
            requirements.configuration,
            capabilities.configuration,
            "configuration",
        ),
        (requirements.logs, capabilities.logs, "logs"),
        (requirements.recovery, capabilities.recovery, "recovery"),
    ];
    checks
        .into_iter()
        .find(|(needed, supported, _)| *needed && !*supported)
        .map(|(_, _, name)| Err(format!("runtime capability unsupported: {name}")))
        .unwrap_or(Ok(()))
}

pub trait RuntimeAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> RuntimeCapabilities;
    fn check_capabilities(&self) -> Result<(), String>;
    #[allow(clippy::too_many_arguments)]
    fn start(
        &self,
        id: &str,
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        log_root: &str,
        work_dir: &Path,
        deadline: Duration,
    ) -> Result<RuntimeObservation, String>;
    fn stop(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String>;
    fn status(&self, id: &str) -> Result<Option<RuntimeObservation>, String>;
    fn adopt(&self, id: &str, identity: ProcessIdentity) -> Result<RuntimeObservation, String>;
    fn logs(&self, id: &str, log_root: &str, lines: usize) -> Result<Vec<String>, String>;
    fn delete(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String>;

    fn shutdown(&self, deadline: Duration) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub executable_device: u64,
    pub executable_inode: u64,
    pub start_ticks: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeObservation {
    pub observed_state: String,
    pub healthy: bool,
    pub identity: Option<ProcessIdentity>,
    pub message: Option<String>,
}

impl RuntimeObservation {
    pub fn healthy(identity: ProcessIdentity) -> Self {
        Self {
            observed_state: "process-healthy".to_string(),
            healthy: true,
            identity: Some(identity),
            message: Some("proved process identity is running".to_string()),
        }
    }

    pub fn absent(message: impl Into<String>) -> Self {
        Self {
            observed_state: "process-absent".to_string(),
            healthy: false,
            identity: None,
            message: Some(message.into()),
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            observed_state: "process-unhealthy".to_string(),
            healthy: false,
            identity: None,
            message: Some(message.into()),
        }
    }

    #[cfg(test)]
    pub fn pid(&self) -> Option<u32> {
        self.identity.as_ref().map(|identity| identity.pid)
    }

    pub fn to_json(&self) -> serde_json::Value {
        let identity = self.identity.as_ref().map(|value| {
            serde_json::json!({
                "pid": value.pid,
                "executableDevice": value.executable_device,
                "executableInode": value.executable_inode,
                "startTicks": value.start_ticks,
            })
        });
        serde_json::json!({
            "observedState": self.observed_state,
            "healthy": self.healthy,
            "identity": identity,
            "message": self.message,
        })
    }

    pub fn identity_from_json(value: &serde_json::Value) -> Option<ProcessIdentity> {
        let identity = value.get("identity")?;
        Some(ProcessIdentity {
            pid: u32::try_from(identity.get("pid")?.as_u64()?).ok()?,
            executable_device: identity.get("executableDevice")?.as_u64()?,
            executable_inode: identity.get("executableInode")?.as_u64()?,
            start_ticks: identity.get("startTicks")?.as_u64()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_capability_pass() {
        let capabilities = RuntimeCapabilities {
            process_identity: false,
            readiness: true,
            storage: true,
            secrets: false,
            configuration: false,
            logs: true,
            recovery: true,
        };
        let result = require(
            capabilities,
            RuntimeRequirements {
                secrets: true,
                ..RuntimeRequirements::default()
            },
        );
        assert_eq!(
            result,
            Err("runtime capability unsupported: secrets".to_string())
        );
    }
}
