use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Json {
        message: String,
    },
    InvalidField {
        field: &'static str,
        message: String,
    },
}

impl ConfigError {
    pub fn json(message: impl Into<String>) -> Self {
        Self::Json {
            message: message.into(),
        }
    }

    pub fn invalid(field: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidField {
            field,
            message: message.into(),
        }
    }

    pub fn field(&self) -> Option<&'static str> {
        match self {
            Self::Json { .. } => None,
            Self::InvalidField { field, .. } => Some(field),
        }
    }
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json { message } => write!(formatter, "invalid JSON: {message}"),
            Self::InvalidField { field, message } => write!(formatter, "{field}: {message}"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdError {
    Invalid { kind: &'static str, value: String },
}

impl IdError {
    pub fn invalid(kind: &'static str, value: impl Into<String>) -> Self {
        Self::Invalid {
            kind,
            value: value.into(),
        }
    }
}

impl Display for IdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid { kind, value } => write!(formatter, "invalid {kind}: {value}"),
        }
    }
}

impl std::error::Error for IdError {}
