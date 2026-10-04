use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use crate::system_message::SystemMessage;
pub type Result<T> = std::result::Result<T, Error>;
pub struct Error {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: SystemMessage,
}
impl Error {
    pub fn invalid(message: impl Into<SystemMessage>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_request",
            message: message.into(),
        }
    }
    pub fn conflict(message: impl Into<SystemMessage>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "conflict",
            message: message.into(),
        }
    }
    pub fn forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "forbidden",
            message: SystemMessage::new("error.forbidden"),
        }
    }
    pub fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "login_required",
            message: SystemMessage::new("error.login_required"),
        }
    }
    pub fn missing() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: SystemMessage::new("error.not_found"),
        }
    }
    pub fn unavailable(message: impl Into<SystemMessage>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "unavailable",
            message: message.into(),
        }
    }
    pub fn internal(e: impl std::fmt::Display) -> Self {
        let reference = uuid::Uuid::new_v4();
        tracing::error!(%reference,error=%e,"operation failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: SystemMessage::new("error.internal").with("reference", reference.to_string()),
        }
    }
}
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for Error {}
impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        if let Some(d) = e.as_database_error() {
            if matches!(d.code().as_deref(), Some("23505" | "23P01")) {
                return Self::conflict(
                    "This already exists or conflicts with another action. Refresh and check again.",
                );
            }
            if matches!(d.code().as_deref(), Some("23503" | "23514" | "22003")) {
                return Self::invalid(
                    "The current state or supplied value does not meet the requirements.",
                );
            }
        }
        Self::internal(e)
    }
}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({"error":{"code":self.code,"message":self.message}})),
        )
            .into_response()
    }
}
