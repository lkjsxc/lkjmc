use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;

pub async fn require_bearer(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    if crate::http_auth::authorized_header(header, state.http_token().as_deref()) {
        next.run(request).await
    } else {
        (StatusCode::FORBIDDEN, "{\"ok\":false}").into_response()
    }
}
