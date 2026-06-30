use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;

pub struct WebReply {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
}

pub fn handle(request: &str, state: &AppState) -> Option<WebReply> {
    let (method, path) = request_line(request)?;
    if !path.starts_with("/web") {
        return None;
    }
    Some(match (method, path) {
        ("GET", "/web") | ("GET", "/web/") => page("lkjmc web", status_page(state)),
        ("GET", "/web/instances") => {
            page("instances", command_page(state, "instance.list", json!({})))
        }
        ("GET", "/web/audit") => page(
            "audit",
            command_page(state, "audit.tail", json!({"lines": 50})),
        ),
        ("GET", "/web/security/token") => page(
            "token",
            command_page(state, "security.daemon-token.status", json!({})),
        ),
        ("POST", value) if value.starts_with("/web/security/token/rotate") => page(
            "token rotate",
            command_page(state, "security.daemon-token.rotate", json!({})),
        ),
        ("POST", value) if value.starts_with("/web/instances/") => instance_action(state, value),
        ("GET", "/web/static/style.css") => WebReply {
            status: 200,
            content_type: "text/css",
            body: "body{font-family:sans-serif}pre{white-space:pre-wrap}".into(),
        },
        _ => WebReply {
            status: 404,
            content_type: "text/plain",
            body: "not found".into(),
        },
    })
}

fn status_page(state: &AppState) -> String {
    let status = dispatch(state, "status", json!({}));
    let doctor = dispatch(state, "doctor", json!({}));
    format!(
        "<h1>lkjmc</h1><h2>Status</h2>{}<h2>Doctor</h2>{}",
        render(status),
        render(doctor)
    )
}

fn command_page(state: &AppState, command: &str, body: Value) -> String {
    render(dispatch(state, command, body))
}

fn instance_action(state: &AppState, path: &str) -> WebReply {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if parts.len() != 4 {
        return WebReply {
            status: 400,
            content_type: "text/plain",
            body: "invalid instance action".into(),
        };
    }
    let command = match parts[3] {
        "start" => "instance.start",
        "stop" => "instance.stop",
        "restart" => "instance.restart",
        _ => {
            return WebReply {
                status: 404,
                content_type: "text/plain",
                body: "unknown action".into(),
            }
        }
    };
    page(
        "instance action",
        command_page(state, command, json!({"id": parts[2]})),
    )
}

fn page(title: &str, body: String) -> WebReply {
    WebReply {
        status: 200,
        content_type: "text/html; charset=utf-8",
        body: format!("<!doctype html><title>{}</title><link rel=stylesheet href=/web/static/style.css><main>{}</main>", escape(title), body),
    }
}

fn render(response: CommandResponse) -> String {
    if response.ok {
        let body = response.body.unwrap_or_else(|| json!({}));
        format!(
            "<pre>{}</pre>",
            escape(&serde_json::to_string_pretty(&body).unwrap_or_default())
        )
    } else {
        let message = response
            .error
            .map(|e| format!("{}: {}", e.code, e.message))
            .unwrap_or_default();
        format!("<p class=error>{}</p>", escape(&message))
    }
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

fn request_line(request: &str) -> Option<(&str, &str)> {
    let mut parts = request.lines().next()?.split_whitespace();
    Some((parts.next()?, parts.next()?))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_status_without_secrets() -> Result<(), String> {
        let state = AppState::with_config_path(
            None,
            "/c".into(),
            "/l".into(),
            "/j".into(),
            "/d".into(),
            None,
            None,
            None,
        );
        let Some(reply) = handle("GET /web HTTP/1.1\r\n\r\n", &state) else {
            return Err("web reply missing".to_string());
        };
        assert_eq!(reply.status, 200);
        assert!(reply.body.contains("Status"));
        assert!(!reply.body.contains("Authorization"));
        Ok(())
    }
}
