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
}
