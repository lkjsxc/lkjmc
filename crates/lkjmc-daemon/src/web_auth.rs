use crate::app::AppState;
use crate::web_api::{page, reply, WebReply};
use crate::web_html::login_form;
use crate::web_request::WebRequest;

pub struct WebAuth {
    pub ok: bool,
    pub bearer: bool,
    pub session_id: Option<String>,
    pub csrf: Option<String>,
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
            response.headers.push((
                "set-cookie",
                format!("lkjmc_session={session_id}; HttpOnly; SameSite=Lax; Path=/web"),
            ));
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

pub fn authorize(raw: &str, state: &AppState, request: &WebRequest) -> WebAuth {
    if crate::http_auth::authorized(raw, state.http_token().as_deref()) {
        return WebAuth {
            ok: true,
            bearer: true,
            session_id: None,
            csrf: None,
        };
    }
    let session_id = request.cookie("lkjmc_session");
    let csrf = state.http_token().as_deref().and_then(|token| {
        session_id
            .as_deref()
            .and_then(|id| state.web_sessions.verify(id, token))
    });
    WebAuth {
        ok: csrf.is_some(),
        bearer: false,
        session_id,
        csrf,
    }
}

pub fn csrf_allowed(request: &WebRequest, auth: &WebAuth) -> bool {
    if auth.bearer {
        return true;
    }
    let Some(csrf) = auth.csrf.as_deref() else {
        return false;
    };
    request.form_value("_csrf").as_deref() == Some(csrf)
        || request.header("x-csrf-token") == Some(csrf)
}
