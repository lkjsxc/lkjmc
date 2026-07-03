use axum::http::HeaderMap;

use crate::app::AppState;
use crate::web::api::{handle_request, WebReply};
use crate::web::request::WebRequest;

#[test]
fn web_login_session_and_csrf_gate_forms() -> Result<(), String> {
    let state = test_state("secret");
    let denied = web_reply("GET /web HTTP/1.1\r\n\r\n", &state)?;
    assert_eq!(denied.status, 403);

    let login = web_reply(
        "POST /web/login HTTP/1.1\r\ncontent-length: 15\r\n\r\npassword=secret",
        &state,
    )?;
    assert_eq!(login.status, 200);
    let cookie = session_cookie(&login)?;
    let csrf = hidden_csrf(&login.body)?;

    let ok = web_reply(
        &format!("GET /web HTTP/1.1\r\nCookie: {cookie}\r\n\r\n"),
        &state,
    )?;
    assert_eq!(ok.status, 200);
    assert!(ok.body.contains("Status"));

    let blocked = web_reply(
        &format!("POST /web/logout HTTP/1.1\r\nCookie: {cookie}\r\ncontent-length: 0\r\n\r\n"),
        &state,
    )?;
    assert_eq!(blocked.status, 403);

    let body = format!("_csrf={csrf}");
    let logout = web_reply(
        &format!(
            "POST /web/logout HTTP/1.1\r\nCookie: {cookie}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        ),
        &state,
    )?;
    assert_eq!(logout.status, 200);
    Ok(())
}

#[test]
fn token_rotation_invalidates_browser_session() -> Result<(), String> {
    let state = test_state("old");
    let login = web_reply(
        "POST /web/login HTTP/1.1\r\ncontent-length: 12\r\n\r\npassword=old",
        &state,
    )?;
    let cookie = session_cookie(&login)?;
    state.set_http_token("new".into())?;
    let reply = web_reply(
        &format!("GET /web HTTP/1.1\r\nCookie: {cookie}\r\n\r\n"),
        &state,
    )?;
    assert_eq!(reply.status, 403);
    Ok(())
}

#[test]
fn bearer_api_mutation_does_not_need_cookie_csrf() -> Result<(), String> {
    let state = test_state("token");
    let reply = web_reply(
        "POST /web/api/security/token/rotate HTTP/1.1\r\nAuthorization: Bearer token\r\ncontent-length: 0\r\n\r\n",
        &state,
    )?;
    assert_eq!(reply.status, 200);
    assert_eq!(reply.content_type, "application/json");
    Ok(())
}

fn test_state(token: &str) -> AppState {
    AppState::with_config_path(
        None,
        8,
        "/config".into(),
        "/log".into(),
        "/jars".into(),
        "/data".into(),
        None,
        None,
        Some(token.into()),
    )
}

fn web_reply(raw: &str, state: &AppState) -> Result<WebReply, String> {
    let request = request(raw)?;
    handle_request(&request, state).ok_or_else(|| "web reply missing".to_string())
}

fn request(raw: &str) -> Result<WebRequest, String> {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let mut lines = head.lines();
    let first = lines
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let mut parts = first.split_whitespace();
    let method = parts.next().ok_or_else(|| "missing method".to_string())?;
    let path = parts.next().ok_or_else(|| "missing path".to_string())?;
    let mut headers = HeaderMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(
                axum::http::HeaderName::from_bytes(name.trim().as_bytes())
                    .map_err(|error| error.to_string())?,
                value
                    .trim()
                    .parse()
                    .map_err(|_| "invalid header".to_string())?,
            );
        }
    }
    Ok(WebRequest::new(method, path, &headers, body.to_string()))
}

fn session_cookie(reply: &WebReply) -> Result<String, String> {
    let header = reply
        .headers
        .iter()
        .find(|(name, _)| *name == "set-cookie")
        .map(|(_, value)| value.as_str())
        .ok_or_else(|| "set-cookie missing".to_string())?;
    header
        .split(';')
        .next()
        .map(ToString::to_string)
        .ok_or_else(|| "session cookie missing".to_string())
}

fn hidden_csrf(body: &str) -> Result<String, String> {
    let marker = "name=_csrf value=\"";
    let start = body
        .find(marker)
        .map(|index| index + marker.len())
        .ok_or_else(|| "csrf marker missing".to_string())?;
    let end = body[start..]
        .find('"')
        .map(|index| index + start)
        .ok_or_else(|| "csrf end missing".to_string())?;
    Ok(body[start..end].to_string())
}
