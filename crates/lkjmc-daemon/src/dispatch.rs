use std::collections::BTreeMap;
use std::sync::OnceLock;

use lkjmc_core::command::{CommandEnvelope, CommandErrorBody, CommandResponse};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::authz::AuthenticatedSubject;

pub type Handler = fn(&AppState, CommandEnvelope) -> CommandResponse;

#[derive(Clone, Copy)]
pub struct Registration {
    pub name: &'static str,
    pub handler: Handler,
}

static DISPATCH: OnceLock<BTreeMap<&'static str, Handler>> = OnceLock::new();

#[cfg(test)]
pub fn dispatch(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    dispatch_as(state, request, AuthenticatedSubject::root("internal"))
}

pub fn dispatch_as(
    state: &AppState,
    request: CommandEnvelope,
    subject: AuthenticatedSubject,
) -> CommandResponse {
    let command_name = request.command.clone();
    let Some(handler) = dispatch_map().get(command_name.as_str()) else {
        return error(
            request,
            "command.unknown",
            format!("Unknown command: {command_name}"),
            false,
        );
    };
    let permission = crate::authz::required(&command_name).unwrap_or(command_name.as_str());
    if let Some(response) = crate::authz::enforce(state, &request, permission, &subject) {
        return response;
    }
    handler(state, request)
}

pub fn registrations() -> &'static [Registration] {
    crate::commands::command_registrations::REGISTRATIONS
}

fn dispatch_map() -> &'static BTreeMap<&'static str, Handler> {
    DISPATCH.get_or_init(|| {
        registrations()
            .iter()
            .map(|entry| (entry.name, entry.handler))
            .collect()
    })
}

pub(crate) fn audit_tail(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let limit = request
        .body
        .get("lines")
        .and_then(Value::as_i64)
        .unwrap_or(100)
        .clamp(1, 500);
    let Some(_database_url) = state.database_url() else {
        return error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    };
    let mut client = match state.database_connection() {
        Ok(client) => client,
        Err(error_value) => return error(request, "database.error", error_value, false),
    };
    match lkjmc_store::audit::tail(&mut client, limit).map(|rows| {
        rows.into_iter()
            .map(|row| {
                json!({
                    "actorKind": row.actor_kind,
                    "actorName": row.actor_name,
                    "action": row.action,
                    "targetKind": row.target_kind,
                    "targetId": row.target_id,
                    "result": row.result
                })
            })
            .collect::<Vec<Value>>()
    }) {
        Ok(events) => ok(request, json!({"events": events})),
        Err(error_value) => error(request, "database.error", error_value.to_string(), false),
    }
}

pub fn ok(request: CommandEnvelope, body: Value) -> CommandResponse {
    CommandResponse {
        request_id: request.request_id,
        ok: true,
        body: Some(body),
        error: None,
    }
}

pub fn error(
    request: CommandEnvelope,
    code: &str,
    message: impl Into<String>,
    retryable: bool,
) -> CommandResponse {
    CommandResponse {
        request_id: request.request_id,
        ok: false,
        body: None,
        error: Some(CommandErrorBody {
            code: code.to_string(),
            message: message.into(),
            retryable,
        }),
    }
}
