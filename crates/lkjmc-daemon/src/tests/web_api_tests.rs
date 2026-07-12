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
    let renewed = set_cookie(&ok)?;
    assert!(renewed.contains("Max-Age="));
    assert!(renewed.starts_with(&cookie));

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
fn expired_browser_session_cannot_mutate() -> Result<(), String> {
    let state = test_state("secret");
    let login = web_reply(
        "POST /web/login HTTP/1.1\r\ncontent-length: 15\r\n\r\npassword=secret",
        &state,
    )?;
    let cookie = session_cookie(&login)?;
    let session_id = cookie
        .strip_prefix("lkjmc_session=")
        .ok_or_else(|| "session prefix".to_string())?;
    let csrf = hidden_csrf(&login.body)?;
    state.web_sessions.expire_for_test(session_id);
    let body = format!("_csrf={csrf}");
    let reply = web_reply(
        &format!(
            "POST /web/logout HTTP/1.1\r\nCookie: {cookie}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        ),
        &state,
    )?;
    assert_eq!(reply.status, 403);
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
    state.set_web_bootstrap("new".into())?;
    let reply = web_reply(
        &format!("GET /web HTTP/1.1\r\nCookie: {cookie}\r\n\r\n"),
        &state,
    )?;
    assert_eq!(reply.status, 403);
    Ok(())
}

#[test]
fn secure_cookie_is_set_for_https_proxy() -> Result<(), String> {
    let state = test_state("secret");
    let login = web_reply(
        "POST /web/login HTTP/1.1\r\nx-forwarded-proto: https\r\ncontent-length: 15\r\n\r\npassword=secret",
        &state,
    )?;
    let header = login
        .headers
        .iter()
        .find(|(name, _)| *name == "set-cookie")
        .map(|(_, value)| value.as_str())
        .ok_or_else(|| "cookie".to_string())?;
    assert!(header.contains("Secure"));
    assert!(header.contains("Max-Age="));
    Ok(())
}

#[test]
fn bootstrap_secret_is_not_a_web_bearer_credential() -> Result<(), String> {
    let state = test_state("token");
    let reply = web_reply(
        "POST /web/api/security/token/rotate HTTP/1.1\r\nAuthorization: Bearer token\r\ncontent-length: 0\r\n\r\n",
        &state,
    )?;
    assert_eq!(reply.status, 403);
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
    handle_request(&request, state)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "web reply missing".to_string())
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
    Ok(WebRequest::new(
        method,
        path,
        &headers,
        body.to_string(),
        None,
    ))
}

fn session_cookie(reply: &WebReply) -> Result<String, String> {
    set_cookie(reply)?
        .split(';')
        .next()
        .map(ToString::to_string)
        .ok_or_else(|| "session cookie missing".to_string())
}

fn set_cookie(reply: &WebReply) -> Result<&str, String> {
    reply
        .headers
        .iter()
        .find(|(name, _)| *name == "set-cookie")
        .map(|(_, value)| value.as_str())
        .ok_or_else(|| "set-cookie missing".to_string())
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
