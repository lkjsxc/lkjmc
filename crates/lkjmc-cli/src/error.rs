use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CliError {
    Message(String),
}

impl CliError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        Self::message(value.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(value: serde_json::Error) -> Self {
        Self::message(value.to_string())
    }
}

impl From<lkjmc_core::error::ConfigError> for CliError {
    fn from(value: lkjmc_core::error::ConfigError) -> Self {
        Self::message(value.to_string())
    }
}

impl From<lkjmc_store::error::StoreError> for CliError {
    fn from(value: lkjmc_store::error::StoreError) -> Self {
        Self::message(value.to_string())
    }
}

impl From<lkjmc_core::error::IdError> for CliError {
    fn from(value: lkjmc_core::error::IdError) -> Self {
        Self::message(value.to_string())
    }
}
