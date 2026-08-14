use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::app::{AppState, BlockingError, RequestAdmission};

pub async fn require_credential(
    State(state): State<AppState>,
    admission: Option<axum::extract::Extension<RequestAdmission>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let credential = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(crate::support::http_auth::bearer_credential)
        .map(ToString::to_string);
    let Some(credential) = credential else {
        return denied();
    };
    let Some(axum::extract::Extension(admission)) = admission else {
        return unavailable();
    };
    let result = admission
        .run_blocking(move || authenticate(&state, &credential))
        .await;
    let subject = match result {
        Ok(Authentication::Subject(subject)) => subject,
        Ok(Authentication::Denied) => return denied(),
        Ok(Authentication::Unavailable) => return unavailable(),
        Ok(Authentication::Deadline) | Err(BlockingError::Deadline) => return deadline(),
        Err(BlockingError::Join) => return unavailable(),
    };
    request.extensions_mut().insert(subject);
    next.run(request).await
}

enum Authentication {
    Subject(crate::authz::AuthenticatedSubject),
    Denied,
    Deadline,
    Unavailable,
}

fn authenticate(state: &AppState, credential: &str) -> Authentication {
    match classify(state.authenticate_credential(credential)) {
        Authentication::Denied => {
            crate::security_audit::denial(state, "tcp", "credential-denied");
            Authentication::Denied
        }
        Authentication::Unavailable => {
            crate::security_audit::denial(state, "tcp", "credential-unavailable");
            Authentication::Unavailable
        }
        authentication => authentication,
    }
}

fn classify(
    result: Result<Option<crate::authz::AuthenticatedSubject>, lkjmc_store::error::StoreError>,
) -> Authentication {
    match result {
        Ok(Some(subject)) => Authentication::Subject(subject),
        Ok(None) => Authentication::Denied,
        Err(error) if error.is_deadline() => Authentication::Deadline,
        Err(_) => Authentication::Unavailable,
    }
}

fn denied() -> Response {
    (
        StatusCode::FORBIDDEN,
        "{\"ok\":false,\"error\":{\"code\":\"auth.denied\"}}",
    )
        .into_response()
}

fn deadline() -> Response {
    (
        StatusCode::REQUEST_TIMEOUT,
        "{\"ok\":false,\"error\":{\"code\":\"command.deadline_exceeded\"}}",
    )
        .into_response()
}

fn unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "{\"ok\":false,\"error\":{\"code\":\"auth.unavailable\"}}",
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlstate_deadline_is_not_auth_denied() {
        for sql_state in [
            postgres::error::SqlState::QUERY_CANCELED,
            postgres::error::SqlState::LOCK_NOT_AVAILABLE,
        ] {
            let error = lkjmc_store::error::StoreError::Postgres {
                message: "deadline test".into(),
                sql_state: Some(sql_state),
            };
            assert!(matches!(classify(Err(error)), Authentication::Deadline));
        }
    }

    #[tokio::test]
    async fn deadline_never_uses_auth_denied() -> Result<(), String> {
        let response = deadline();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .map_err(|error| error.to_string())?;
        assert_eq!(
            body.as_ref(),
            b"{\"ok\":false,\"error\":{\"code\":\"command.deadline_exceeded\"}}"
        );
        Ok(())
    }
}
