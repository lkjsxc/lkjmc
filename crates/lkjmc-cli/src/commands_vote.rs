use serde_json::json;

use crate::args_vote::VoteCommand;
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: VoteCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        VoteCommand::List => daemon_command(
            socket,
            "player.vote.list",
            json!({}),
            json_output,
            "ok vote list",
        ),
        VoteCommand::UpsertLink { id, title_key, url } => daemon_command(
            socket,
            "vote.link.upsert",
            json!({"id": id, "titleKey": title_key, "url": url}),
            json_output,
            "ok vote link upsert",
        ),
    }
}
