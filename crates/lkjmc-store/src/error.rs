use std::fmt::{Display, Formatter};

use postgres::error::SqlState;

#[derive(Debug)]
pub enum StoreError {
    Deadline,
    Postgres {
        message: String,
        sql_state: Option<SqlState>,
    },
    InvalidState(String),
}

impl StoreError {
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::InvalidState(message.into())
    }
}

impl StoreError {
    pub fn is_deadline(&self) -> bool {
        match self {
            Self::Deadline => true,
            Self::Postgres {
                sql_state: Some(code),
                ..
            } => code == &SqlState::QUERY_CANCELED || code == &SqlState::LOCK_NOT_AVAILABLE,
            _ => false,
        }
    }
}

impl From<postgres::Error> for StoreError {
    fn from(value: postgres::Error) -> Self {
        let sql_state = value.as_db_error().map(|error| error.code().clone());
        let message = value
            .as_db_error()
            .map(|error| error.message().to_string())
            .unwrap_or_else(|| value.to_string());
        Self::Postgres { message, sql_state }
    }
}

impl Display for StoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deadline => write!(formatter, "database request deadline elapsed"),
            Self::Postgres { message, .. } => write!(formatter, "postgres error: {message}"),
            Self::InvalidState(message) => write!(formatter, "invalid store state: {message}"),
        }
    }
}

impl std::error::Error for StoreError {}
