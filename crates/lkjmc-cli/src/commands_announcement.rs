use serde_json::json;

use crate::args_announcement::AnnouncementCommand;
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: AnnouncementCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        AnnouncementCommand::Send { server_id, message } => daemon_command(
            socket,
            "announcement.create",
            json!({"actorName": "cli", "serverId": server_id, "message": message}),
            json_output,
            "ok announcement send",
        ),
    }
}
