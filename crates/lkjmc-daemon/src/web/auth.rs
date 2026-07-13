use crate::app::AppState;
use crate::web::api::{page, reply, WebReply};
use crate::web::html::login_form;
use crate::web::request::WebRequest;

pub struct WebAuth {
    pub subject: Option<crate::authz::AuthenticatedSubject>,
    pub bearer: bool,
    pub session_id: Option<String>,
    pub csrf: Option<String>,
    pub renewed_cookie: Option<String>,
}

pub fn login(state: &AppState, request: &WebRequest) -> WebReply {
    if !state.web_sessions.allow_login(request.source()) {
        crate::security_audit::denial(state, "web", "login-rate-limited");
        return reply(
            429,
            "text/html; charset=utf-8",
            &login_form(Some("login unavailable")),
        );
    }
    let valid = request
        .form_value("password")
        .is_some_and(|value| state.verify_web_bootstrap(&value));
    let Some(fingerprint) = state.web_bootstrap_fingerprint() else {
        crate::security_audit::denial(state, "web", "bootstrap-unavailable");
        return reply(
            403,
            "text/html; charset=utf-8",
            &login_form(Some("login failed")),
        );
    };
    if !valid {
        crate::security_audit::denial(state, "web", "login-denied");
        return reply(
            403,
            "text/html; charset=utf-8",
            &login_form(Some("login failed")),
        );
    }
    match state.web_sessions.create(&fingerprint) {
        Ok((session_id, csrf)) => {
            state.web_sessions.login_succeeded(request.source());
            let mut response = page("login ok", "<p>login ok</p>".into(), Some(&csrf));
            response
                .headers
                .push(("set-cookie", session_cookie(&session_id, request)));
            response
        }
        Err(_) => {
            crate::security_audit::denial(state, "web", "session-unavailable");
            reply(
                403,
                "text/html; charset=utf-8",
                &login_form(Some("login unavailable")),
            )
        }
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

pub fn authorize(
    state: &AppState,
    request: &WebRequest,
) -> Result<WebAuth, lkjmc_store::error::StoreError> {
    if let Some(credential) = request
        .header("authorization")
        .and_then(crate::support::http_auth::bearer_credential)
    {
        match state.authenticate_credential(credential) {
            Ok(Some(subject)) if subject.surface == "web" => {
                return Ok(authenticated(subject, true, None, None, None));
            }
            Err(error) if error.is_deadline() => return Err(error),
            Ok(_) | Err(_) => {}
        }
    }
    let session_id = request.cookie("lkjmc_session");
    let csrf = state.web_bootstrap_fingerprint().and_then(|fingerprint| {
        session_id
            .as_deref()
            .and_then(|id| state.web_sessions.verify(id, &fingerprint))
    });
    let renewed_cookie = csrf
        .as_ref()
        .and(session_id.as_deref())
        .map(|id| session_cookie(id, request));
    match csrf {
        Some(csrf) => Ok(authenticated(
            crate::authz::AuthenticatedSubject::web_session(),
            false,
            session_id,
            Some(csrf),
            renewed_cookie,
        )),
        None => Ok(WebAuth {
            subject: None,
            bearer: false,
            session_id,
            csrf: None,
            renewed_cookie: None,
        }),
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

fn authenticated(
    subject: crate::authz::AuthenticatedSubject,
    bearer: bool,
    session_id: Option<String>,
    csrf: Option<String>,
    renewed_cookie: Option<String>,
) -> WebAuth {
    WebAuth {
        subject: Some(subject),
        bearer,
        session_id,
        csrf,
        renewed_cookie,
    }
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
