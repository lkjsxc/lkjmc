use lkjmc_core::admin::AdminRole;
use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;

use crate::api;
use crate::app::AppState;

pub fn handle(_state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let command_name = request.command.clone();
    match command_name.as_str() {
        "admin.role.list" => role_list(request),
        command => api::error(
            request,
            "command.unknown",
            format!("Unknown command: {command}"),
            false,
        ),
    }
}

fn role_list(request: CommandEnvelope) -> CommandResponse {
    let roles = AdminRole::all()
        .iter()
        .map(|role| json!({"id": role.id(), "permissions": role.permissions()}))
        .collect::<Vec<_>>();
    api::ok(request, json!({"roles": roles}))
}
