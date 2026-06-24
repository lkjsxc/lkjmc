use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum StoreError {
    Postgres(String),
    InvalidState(String),
}

impl StoreError {
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::InvalidState(message.into())
    }
}

impl From<postgres::Error> for StoreError {
    fn from(value: postgres::Error) -> Self {
        Self::Postgres(value.to_string())
    }
}

impl Display for StoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postgres(message) => write!(formatter, "postgres error: {message}"),
            Self::InvalidState(message) => write!(formatter, "invalid store state: {message}"),
        }
    }
}

impl std::error::Error for StoreError {}
