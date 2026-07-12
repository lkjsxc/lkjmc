use axum::body::Bytes;
use axum::extract::{ConnectInfo, Extension, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::{Json, Router};
use serde_json::json;

use crate::app::{AppState, RequestAdmission};
use crate::web::api::{handle_request, WebReply};
use crate::web::request::WebRequest;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/web", any(axum_handle))
        .route("/web/", any(axum_handle))
        .route("/web/{*path}", any(axum_handle))
        .with_state(state)
}

async fn axum_handle(
    State(state): State<AppState>,
    admission: Option<Extension<RequestAdmission>>,
    peer: Option<Extension<ConnectInfo<std::net::SocketAddr>>>,
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
    let source = peer.map(|Extension(ConnectInfo(peer))| peer.ip().to_string());
    let request = WebRequest::new(method.as_str(), &path, &headers, body, source);
    let Some(Extension(admission)) = admission else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "request admission unavailable",
        )
            .into_response();
    };
    match admission
        .run_blocking(move || handle_request(&request, &state))
        .await
    {
        Ok(Ok(Some(reply))) => into_response(reply),
        Ok(Ok(None)) => (StatusCode::NOT_FOUND, "not found").into_response(),
        Ok(Err(error)) if error.is_deadline() => deadline(),
        Ok(Err(_)) | Err(crate::app::BlockingError::Join) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
        }
        Err(crate::app::BlockingError::Deadline) => deadline(),
    }
}

fn deadline() -> Response {
    (
        StatusCode::REQUEST_TIMEOUT,
        Json(json!({"ok": false, "error": {"code": "command.deadline_exceeded"}})),
    )
        .into_response()
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
    let headers = response.headers_mut();
    headers.insert("cache-control", HeaderValue::from_static("no-store"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'; base-uri 'none'"),
    );
    response
}
