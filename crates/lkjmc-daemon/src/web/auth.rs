use crate::app::AppState;
use crate::web::api::{page, reply, WebReply};
use crate::web::html::login_form;
use crate::web::request::WebRequest;

pub struct WebAuth {
    pub ok: bool,
    pub bearer: bool,
    pub session_id: Option<String>,
    pub csrf: Option<String>,
    pub renewed_cookie: Option<String>,
}

pub fn login(state: &AppState, request: &WebRequest) -> WebReply {
    let Some(token) = state.http_token() else {
        return reply(
            403,
            "text/html; charset=utf-8",
            &login_form(Some("web token not configured")),
        );
    };
    if request.form_value("password").as_deref() != Some(token.as_str()) {
        return reply(
            403,
            "text/html; charset=utf-8",
            &login_form(Some("login failed")),
        );
    }
    match state.web_sessions.create(&token) {
        Ok((session_id, csrf)) => {
            let mut response = page("login ok", "<p>login ok</p>".into(), Some(&csrf));
            response
                .headers
                .push(("set-cookie", session_cookie(&session_id, request)));
            response
        }
        Err(error) => reply(500, "text/plain", &error),
    }
}

pub fn logout(state: &AppState, session_id: Option<&str>) -> WebReply {
    if let Some(id) = session_id {
        state.web_sessions.revoke(id);
    }
    let mut response = page("logout", "<p>logged out</p>".into(), None);
    response.headers.push((
        "set-cookie",
        "lkjmc_session=; Max-Age=0; HttpOnly; SameSite=Lax; Path=/web".into(),
    ));
    response
}

pub fn authorize(state: &AppState, request: &WebRequest) -> WebAuth {
    if crate::support::http_auth::authorized_header(
        request.header("authorization"),
        state.http_token().as_deref(),
    ) {
        return WebAuth {
            ok: true,
            bearer: true,
            session_id: None,
            csrf: None,
            renewed_cookie: None,
        };
    }
    let session_id = request.cookie("lkjmc_session");
    let csrf = state.http_token().as_deref().and_then(|token| {
        session_id
            .as_deref()
            .and_then(|id| state.web_sessions.verify(id, token))
    });
    let renewed_cookie = csrf
        .as_ref()
        .and(session_id.as_deref())
        .map(|id| session_cookie(id, request));
    WebAuth {
        ok: csrf.is_some(),
        bearer: false,
        session_id,
        csrf,
        renewed_cookie,
    }
}

pub fn csrf_allowed(request: &WebRequest, auth: &WebAuth) -> bool {
    if auth.bearer {
        return request.route().starts_with("/web/api/");
    }
    let Some(csrf) = auth.csrf.as_deref() else {
        return false;
    };
    request.form_value("_csrf").as_deref() == Some(csrf)
        || request.header("x-csrf-token") == Some(csrf)
}

fn session_cookie(session_id: &str, request: &WebRequest) -> String {
    let secure = if secure_cookie(request) {
        "; Secure"
    } else {
        ""
    };
    format!(
        "lkjmc_session={session_id}; Max-Age={}; HttpOnly; SameSite=Lax; Path=/web{secure}",
        crate::web::sessions::WebSessions::max_age_seconds()
    )
}

fn secure_cookie(request: &WebRequest) -> bool {
    request.header("x-forwarded-proto") == Some("https")
        || request.header("x-forwarded-ssl") == Some("on")
        || request
            .header("forwarded")
            .is_some_and(|value| value.to_ascii_lowercase().contains("proto=https"))
}
