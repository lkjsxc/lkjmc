use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct OpsError {
    message: String,
}

impl OpsError {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn context(label: &str, error: impl Display) -> Self {
        Self::message(format!("{label}: {error}"))
    }
}

impl Display for OpsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for OpsError {}

impl From<std::io::Error> for OpsError {
    fn from(error: std::io::Error) -> Self {
        Self::context("filesystem operation failed", error)
    }
}

impl From<serde_json::Error> for OpsError {
    fn from(error: serde_json::Error) -> Self {
        Self::context("JSON operation failed", error)
    }
}

pub type Result<T> = std::result::Result<T, OpsError>;
