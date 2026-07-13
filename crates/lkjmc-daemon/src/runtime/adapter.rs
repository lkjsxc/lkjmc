use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

pub trait RuntimeAdapter: Send {
    fn name(&self) -> &'static str;
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
    fn logs(&mut self, id: &str, log_root: &str, lines: usize) -> Result<Vec<String>, String>;
    fn delete(&mut self, id: &str) -> Result<RuntimeObservation, String>;

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
