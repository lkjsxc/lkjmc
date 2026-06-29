use serde_json::json;

use crate::args_admin::{principal, AdminCommand};
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: AdminCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        AdminCommand::RoleList => daemon_command(
            socket,
            "admin.role.list",
            json!({}),
            json_output,
            "ok admin role list",
        ),
        AdminCommand::Grant {
            principal: value,
            role,
            reason,
        } => {
            let (kind, id) = principal(&value);
            daemon_command(
                socket,
                "admin.grant.create",
                json!({
                    "principalKind": kind, "principalId": id, "roleId": role, "reason": reason
                }),
                json_output,
                "ok admin grant",
            )
        }
        AdminCommand::Revoke {
            principal: value,
            role,
            reason,
        } => {
            let (kind, id) = principal(&value);
            daemon_command(
                socket,
                "admin.grant.revoke",
                json!({
                    "principalKind": kind, "principalId": id, "roleId": role, "reason": reason
                }),
                json_output,
                "ok admin revoke",
            )
        }
        AdminCommand::Inspect { principal: value } => {
            let (kind, id) = principal(&value);
            daemon_command(
                socket,
                "admin.principal.inspect",
                json!({
                    "principalKind": kind, "principalId": id
                }),
                json_output,
                "ok admin inspect",
            )
        }
        AdminCommand::Audit { lines } => daemon_command(
            socket,
            "admin.audit.tail",
            json!({"lines": lines}),
            json_output,
            "ok admin audit",
        ),
    }
}
