use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;
use crate::authz::AuthenticatedSubject;

pub async fn require_bearer(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    if crate::support::http_auth::authorized_header(header, state.http_token().as_deref())
        || crate::support::http_auth::authorized_header(
            header,
            state.http_previous_token().as_deref(),
        )
    {
        request
            .extensions_mut()
            .insert(AuthenticatedSubject::root("bearer"));
        return next.run(request).await;
    }
    if let Some(subject) = header
        .and_then(crate::support::http_auth::bearer_credential)
        .and_then(|credential| scoped_subject(&state, credential))
    {
        request.extensions_mut().insert(subject);
        return next.run(request).await;
    }
    (StatusCode::FORBIDDEN, "{\"ok\":false}").into_response()
}

fn scoped_subject(state: &AppState, credential: &str) -> Option<AuthenticatedSubject> {
    if credential.trim().is_empty() || state.database_url().is_none() {
        return None;
    }
    let hash = lkjmc_core::security::token_hash(credential);
    let mut client = state.database_connection().ok()?;
    let record = lkjmc_store::daemon_token::find_active(&mut client, &hash).ok()??;
    Some(AuthenticatedSubject::scoped(
        record.surface,
        record.principal_kind,
        record.principal_id,
        record.scopes,
    ))
}
