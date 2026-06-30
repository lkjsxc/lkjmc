use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCapabilities {
    pub start: bool,
    pub stop: bool,
    pub restart: bool,
    pub delete: bool,
    pub logs: bool,
    pub recover: bool,
    pub readiness: bool,
}

impl RuntimeCapabilities {
    pub fn local_process() -> Self {
        Self {
            start: true,
            stop: true,
            restart: true,
            delete: true,
            logs: true,
            recover: true,
            readiness: true,
        }
    }
}

pub trait RuntimeAdapter: Send {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> RuntimeCapabilities;
    fn recover(&mut self, id: &str, pid: u32) -> RuntimeObservation;
    fn start(
        &mut self,
        id: &str,
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        log_root: &str,
        work_dir: &Path,
    ) -> Result<RuntimeObservation, String>;
    fn stop(&mut self, id: &str, timeout: Duration) -> Result<RuntimeObservation, String>;
    fn status(&mut self, id: &str) -> Result<Option<RuntimeObservation>, String>;

    fn is_running(&mut self, id: &str) -> Result<bool, String> {
        Ok(self.status(id)?.is_some_and(|status| status.healthy))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeObservation {
    pub observed_state: String,
    pub healthy: bool,
    pub pid: Option<u32>,
    pub message: Option<String>,
}

impl RuntimeObservation {
    pub fn healthy(pid: u32) -> Self {
        Self {
            observed_state: "process-healthy".to_string(),
            healthy: true,
            pid: Some(pid),
            message: Some("process running".to_string()),
        }
    }

    pub fn absent(message: impl Into<String>) -> Self {
        Self {
            observed_state: "process-absent".to_string(),
            healthy: false,
            pid: None,
            message: Some(message.into()),
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            observed_state: "process-unhealthy".to_string(),
            healthy: false,
            pid: None,
            message: Some(message.into()),
        }
    }
}
