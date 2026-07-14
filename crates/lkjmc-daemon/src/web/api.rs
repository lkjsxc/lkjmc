use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::authz::AuthenticatedSubject;
use crate::web::html::{escape, login_form, render};
use crate::web::request::WebRequest;
pub use crate::web::response::WebReply;
pub(crate) use crate::web::response::{page, reply};

pub fn handle_request(
    request: &WebRequest,
    state: &AppState,
) -> Result<Option<WebReply>, lkjmc_store::error::StoreError> {
    let route = request.route();
    if !route.starts_with("/web") {
        return Ok(None);
    }
    if route == "/web/static/style.css" {
        return Ok(Some(reply(
            200,
            "text/css",
            "body{font-family:sans-serif}pre{white-space:pre-wrap}",
        )));
    }
    if request.method == "GET" && route == "/web/login" {
        return Ok(Some(page("login", login_form(None), None)));
    }
    if request.method == "POST" && route == "/web/login" {
        return Ok(Some(crate::web::auth::login(state, request)));
    }
    let auth = crate::web::auth::authorize(state, request)?;
    let Some(subject) = auth.subject.clone() else {
        return Ok(Some(reply(
            403,
            "text/html; charset=utf-8",
            &login_form(Some("login required")),
        )));
    };
    if request.method == "POST" && !crate::web::auth::csrf_allowed(request, &auth) {
        return Ok(Some(reply(403, "text/plain", "csrf token required")));
    }
    let mut response = match (request.method.as_str(), route) {
        ("POST", "/web/logout") => crate::web::auth::logout(state, auth.session_id.as_deref()),
        ("GET", "/web") | ("GET", "/web/") => page(
            "lkjmc web",
            status_page(state, &subject),
            auth.csrf.as_deref(),
        ),
        ("GET", "/web/instances") => page(
            "instances",
            command_page(state, &subject, "instance.list", json!({})),
            auth.csrf.as_deref(),
        ),
        ("GET", "/web/audit") => page(
            "audit",
            command_page(state, &subject, "audit.tail", json!({"lines": 50})),
            auth.csrf.as_deref(),
        ),
        ("GET", "/web/observability") => {
            crate::web::observability::view(state, auth.csrf.as_deref())
        }
        ("POST", "/web/support-bundle") => {
            crate::web::observability::bundle(state, auth.csrf.as_deref())
        }
        ("GET", "/web/security/token") => page(
            "token",
            token_page(state, &subject, auth.csrf.as_deref()),
            auth.csrf.as_deref(),
        ),
        ("POST", value) if value.starts_with("/web/security/token/rotate") => page(
            "token rotate",
            command_page(state, &subject, "security.daemon-token.rotate", json!({})),
            auth.csrf.as_deref(),
        ),
        ("POST", value) if value.starts_with("/web/instances/") => {
            instance_action(state, &subject, value, auth.csrf.as_deref())
        }
        ("GET", "/web/api/status") => command_json(state, &subject, "status", json!({})),
        ("GET", "/web/api/instances") => command_json(state, &subject, "instance.list", json!({})),
        ("POST", "/web/api/security/token/rotate") => {
            command_json(state, &subject, "security.daemon-token.rotate", json!({}))
        }
        _ => reply(404, "text/plain", "not found"),
    };
    if route != "/web/logout" {
        if let Some(cookie) = auth.renewed_cookie {
            response.headers.push(("set-cookie", cookie));
        }
    }
    Ok(Some(response))
}

fn status_page(state: &AppState, subject: &AuthenticatedSubject) -> String {
    format!(
        "<h1>lkjmc</h1><h2>Status</h2>{}<h2>Doctor</h2>{}",
        render(dispatch(state, subject, "status", json!({}))),
        render(dispatch(state, subject, "doctor", json!({})))
    )
}

fn token_page(state: &AppState, subject: &AuthenticatedSubject, csrf: Option<&str>) -> String {
    let form = csrf.map(|value| format!("<form method=post action=/web/security/token/rotate><input type=hidden name=_csrf value=\"{}\"><button>rotate</button></form>", escape(value))).unwrap_or_default();
    format!(
        "{}{}",
        command_page(state, subject, "security.daemon-token.status", json!({})),
        form
    )
}

fn command_page(
    state: &AppState,
    subject: &AuthenticatedSubject,
    command: &str,
    body: Value,
) -> String {
    render(dispatch(state, subject, command, body))
}

fn command_json(
    state: &AppState,
    subject: &AuthenticatedSubject,
    command: &str,
    body: Value,
) -> WebReply {
    let response = dispatch(state, subject, command, body);
    let status = if response
        .error
        .as_ref()
        .is_some_and(|error| error.code == "command.deadline_exceeded")
    {
        408
    } else {
        200
    };
    let encoded = serde_json::to_string(&response).unwrap_or_default();
    reply(status, "application/json", &encoded)
}

fn instance_action(
    state: &AppState,
    subject: &AuthenticatedSubject,
    path: &str,
    csrf: Option<&str>,
) -> WebReply {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if parts.len() != 4 {
        return reply(400, "text/plain", "invalid instance action");
    }
    let command = match parts[3] {
        "start" => "instance.start",
        "stop" => "instance.stop",
        "restart" => "instance.restart",
        _ => return reply(404, "text/plain", "unknown action"),
    };
    page(
        "instance action",
        command_page(state, subject, command, json!({"id": parts[2]})),
        csrf,
    )
}

fn dispatch(
    state: &AppState,
    subject: &AuthenticatedSubject,
    command: &str,
    body: Value,
) -> CommandResponse {
    crate::dispatch::dispatch_as(
        state,
        CommandEnvelope {
            request_id: CommandId::parse("request id", Uuid::new_v4().to_string())
                .unwrap_or_else(|_| CommandId::internal("web-request")),
            actor: Actor {
                kind: ActorKind::WebOperator,
                name: "untrusted-web-envelope".into(),
            },
            command: command.into(),
            body,
        },
        subject.clone(),
    )
}
