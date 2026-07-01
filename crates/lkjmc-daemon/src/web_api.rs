use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::web_html::{escape, login_form, render};
use crate::web_request::WebRequest;

pub struct WebReply {
    pub status: u16,
    pub content_type: &'static str,
    pub headers: Vec<(&'static str, String)>,
    pub body: String,
}

pub fn handle(raw: &str, state: &AppState) -> Option<WebReply> {
    let request = WebRequest::parse(raw)?;
    let route = request.route();
    if !route.starts_with("/web") {
        return None;
    }
    if route == "/web/static/style.css" {
        return Some(reply(
            200,
            "text/css",
            "body{font-family:sans-serif}pre{white-space:pre-wrap}",
        ));
    }
    if request.method == "GET" && route == "/web/login" {
        return Some(page("login", login_form(None), None));
    }
    if request.method == "POST" && route == "/web/login" {
        return Some(crate::web_auth::login(state, &request));
    }
    let auth = crate::web_auth::authorize(raw, state, &request);
    if !auth.ok {
        return Some(reply(
            403,
            "text/html; charset=utf-8",
            &login_form(Some("login required")),
        ));
    }
    if request.method == "POST" && !crate::web_auth::csrf_allowed(&request, &auth) {
        return Some(reply(403, "text/plain", "csrf token required"));
    }
    Some(match (request.method.as_str(), route) {
        ("POST", "/web/logout") => crate::web_auth::logout(state, auth.session_id.as_deref()),
        ("GET", "/web") | ("GET", "/web/") => {
            page("lkjmc web", status_page(state), auth.csrf.as_deref())
        }
        ("GET", "/web/instances") => page(
            "instances",
            command_page(state, "instance.list", json!({})),
            auth.csrf.as_deref(),
        ),
        ("GET", "/web/audit") => page(
            "audit",
            command_page(state, "audit.tail", json!({"lines": 50})),
            auth.csrf.as_deref(),
        ),
        ("GET", "/web/security/token") => page(
            "token",
            token_page(state, auth.csrf.as_deref()),
            auth.csrf.as_deref(),
        ),
        ("POST", value) if value.starts_with("/web/security/token/rotate") => page(
            "token rotate",
            command_page(state, "security.daemon-token.rotate", json!({})),
            auth.csrf.as_deref(),
        ),
        ("POST", value) if value.starts_with("/web/instances/") => {
            instance_action(state, value, auth.csrf.as_deref())
        }
        ("GET", "/web/api/status") => command_json(state, "status", json!({})),
        ("GET", "/web/api/instances") => command_json(state, "instance.list", json!({})),
        ("POST", "/web/api/security/token/rotate") => {
            command_json(state, "security.daemon-token.rotate", json!({}))
        }
        _ => reply(404, "text/plain", "not found"),
    })
}

fn status_page(state: &AppState) -> String {
    format!(
        "<h1>lkjmc</h1><h2>Status</h2>{}<h2>Doctor</h2>{}",
        render(dispatch(state, "status", json!({}))),
        render(dispatch(state, "doctor", json!({})))
    )
}

fn token_page(state: &AppState, csrf: Option<&str>) -> String {
    let form = csrf.map(|value| format!("<form method=post action=/web/security/token/rotate><input type=hidden name=_csrf value=\"{}\"><button>rotate</button></form>", escape(value))).unwrap_or_default();
    format!(
        "{}{}",
        command_page(state, "security.daemon-token.status", json!({})),
        form
    )
}

fn command_page(state: &AppState, command: &str, body: Value) -> String {
    render(dispatch(state, command, body))
}

fn command_json(state: &AppState, command: &str, body: Value) -> WebReply {
    let encoded = serde_json::to_string(&dispatch(state, command, body)).unwrap_or_default();
    reply(200, "application/json", &encoded)
}

fn instance_action(state: &AppState, path: &str, csrf: Option<&str>) -> WebReply {
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
        command_page(state, command, json!({"id": parts[2]})),
        csrf,
    )
}

pub(crate) fn page(title: &str, body: String, csrf: Option<&str>) -> WebReply {
    let logout = csrf.map(|value| format!("<form method=post action=/web/logout><input type=hidden name=_csrf value=\"{}\"><button>logout</button></form>", escape(value))).unwrap_or_default();
    reply(200, "text/html; charset=utf-8", &format!("<!doctype html><title>{}</title><link rel=stylesheet href=/web/static/style.css><main>{logout}{body}</main>", escape(title)))
}

fn dispatch(state: &AppState, command: &str, body: Value) -> CommandResponse {
    crate::api::dispatch(
        state,
        CommandEnvelope {
            request_id: CommandId::parse("request id", Uuid::new_v4().to_string())
                .unwrap_or_else(|_| CommandId::internal("web-request")),
            actor: Actor {
                kind: ActorKind::WebOperator,
                name: "web".into(),
            },
            command: command.into(),
            body,
        },
    )
}

pub(crate) fn reply(status: u16, content_type: &'static str, body: &str) -> WebReply {
    WebReply {
        status,
        content_type,
        headers: Vec::new(),
        body: body.to_string(),
    }
}
