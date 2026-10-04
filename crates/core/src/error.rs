use crate::system_message::SystemMessage;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
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
                    "text.this_already_exists_or_conflicts_with_another_action_re_20ba718d67",
                );
            }
            if matches!(d.code().as_deref(), Some("23503" | "23514" | "22003")) {
                return Self::invalid(
                    "text.the_current_state_or_supplied_value_does_not_meet_the_r_1e4eebdf15",
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

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn wire_errors_are_structured_and_do_not_expose_diagnostics() {
        let diagnostic = "内部の日本語診断";
        let response = Error::invalid(diagnostic).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["error"]["message"]["id"], "system.unknown");
        assert!(body["error"]["message"]["params"]["reference"].is_string());
        assert!(!String::from_utf8(bytes.to_vec()).unwrap().contains("内部"));
    }
    #[tokio::test]
    async fn internal_failure_retains_only_a_reference_on_the_wire() {
        let response = Error::internal("private upstream diagnostic").into_response();
        let bytes = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["error"]["message"]["id"], "error.internal");
        assert!(body["error"]["message"]["params"]["reference"].is_string());
        assert!(
            !String::from_utf8(bytes.to_vec())
                .unwrap()
                .contains("upstream")
        );
    }
}
