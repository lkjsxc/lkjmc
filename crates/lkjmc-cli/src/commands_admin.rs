use serde_json::json;

use crate::args_admin::AdminCommand;
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
    }
}
