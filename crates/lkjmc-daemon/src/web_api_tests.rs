use crate::app::AppState;
use crate::web_api::handle;

#[test]
fn web_login_session_and_csrf_gate_forms() {
    let state = test_state("secret");
    let denied = handle("GET /web HTTP/1.1\r\n\r\n", &state).unwrap();
    assert_eq!(denied.status, 403);

    let login = handle(
        "POST /web/login HTTP/1.1\r\ncontent-length: 15\r\n\r\npassword=secret",
        &state,
    )
    .unwrap();
    assert_eq!(login.status, 200);
    let cookie = login
        .headers
        .iter()
        .find(|(name, _)| *name == "set-cookie")
        .map(|(_, value)| value.split(';').next().unwrap().to_string())
        .unwrap();
    let csrf = hidden_csrf(&login.body);

    let ok = handle(
        &format!("GET /web HTTP/1.1\r\nCookie: {cookie}\r\n\r\n"),
        &state,
    )
    .unwrap();
    assert_eq!(ok.status, 200);
    assert!(ok.body.contains("Status"));

    let blocked = handle(
        &format!("POST /web/logout HTTP/1.1\r\nCookie: {cookie}\r\ncontent-length: 0\r\n\r\n"),
        &state,
    )
    .unwrap();
    assert_eq!(blocked.status, 403);

    let body = format!("_csrf={csrf}");
    let logout = handle(
        &format!(
            "POST /web/logout HTTP/1.1\r\nCookie: {cookie}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        ),
        &state,
    )
    .unwrap();
    assert_eq!(logout.status, 200);
}

#[test]
fn token_rotation_invalidates_browser_session() {
    let state = test_state("old");
    let login = handle(
        "POST /web/login HTTP/1.1\r\ncontent-length: 12\r\n\r\npassword=old",
        &state,
    )
    .unwrap();
    let cookie = login.headers[0].1.split(';').next().unwrap().to_string();
    state.set_http_token("new".into()).unwrap();
    let reply = handle(
        &format!("GET /web HTTP/1.1\r\nCookie: {cookie}\r\n\r\n"),
        &state,
    )
    .unwrap();
    assert_eq!(reply.status, 403);
}

#[test]
fn bearer_api_mutation_does_not_need_cookie_csrf() {
    let state = test_state("token");
    let reply = handle(
        "POST /web/api/security/token/rotate HTTP/1.1\r\nAuthorization: Bearer token\r\ncontent-length: 0\r\n\r\n",
        &state,
    )
    .unwrap();
    assert_eq!(reply.status, 200);
    assert_eq!(reply.content_type, "application/json");
}

fn test_state(token: &str) -> AppState {
    AppState::with_config_path(
        None,
        "/config".into(),
        "/log".into(),
        "/jars".into(),
        "/data".into(),
        None,
        None,
        Some(token.into()),
    )
}

fn hidden_csrf(body: &str) -> String {
    let marker = "name=_csrf value=\"";
    let start = body.find(marker).unwrap() + marker.len();
    let end = body[start..].find('"').unwrap() + start;
    body[start..end].to_string()
}
