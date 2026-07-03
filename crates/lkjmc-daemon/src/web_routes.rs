use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::Router;

use crate::app::AppState;
use crate::web_api::{handle_request, WebReply};
use crate::web_request::WebRequest;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/web", any(axum_handle))
        .route("/web/", any(axum_handle))
        .route("/web/{*path}", any(axum_handle))
        .with_state(state)
}

async fn axum_handle(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let path = uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or(uri.path())
        .to_string();
    let body = String::from_utf8_lossy(&body).to_string();
    let request = WebRequest::new(method.as_str(), &path, &headers, body);
    match tokio::task::spawn_blocking(move || handle_request(&request, &state)).await {
        Ok(Some(reply)) => into_response(reply),
        Ok(None) => (StatusCode::NOT_FOUND, "not found").into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

fn into_response(reply: WebReply) -> Response {
    let status = StatusCode::from_u16(reply.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut response = (status, reply.body).into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static(reply.content_type),
    );
    for (name, value) in reply.headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(&value),
        ) {
            response.headers_mut().insert(name, value);
        }
    }
    response
}
