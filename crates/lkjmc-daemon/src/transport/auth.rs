use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;

pub async fn require_credential(
    State(state): State<AppState>,
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
    let subject = tokio::task::spawn_blocking(move || authenticate(&state, &credential))
        .await
        .ok()
        .flatten();
    let Some(subject) = subject else {
        return denied();
    };
    request.extensions_mut().insert(subject);
    next.run(request).await
}

fn authenticate(state: &AppState, credential: &str) -> Option<crate::authz::AuthenticatedSubject> {
    match state.authenticate_credential(credential) {
        Ok(Some(subject)) => Some(subject),
        Ok(None) => {
            crate::security_audit::denial(state, "tcp", "credential-denied");
            None
        }
        Err(()) => {
            crate::security_audit::denial(state, "tcp", "credential-unavailable");
            None
        }
    }
}

fn denied() -> Response {
    (
        StatusCode::FORBIDDEN,
        "{\"ok\":false,\"error\":{\"code\":\"auth.denied\"}}",
    )
        .into_response()
}
