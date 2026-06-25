use serde_json::json;

use crate::args_kit::KitCommand;
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: KitCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        KitCommand::List => daemon_command(
            socket,
            "player.kit.list",
            json!({}),
            json_output,
            "ok kit list",
        ),
        KitCommand::Upsert {
            id,
            title_key,
            reward_points,
            cooldown_hours,
        } => daemon_command(
            socket,
            "kit.upsert",
            json!({
                "kitId": id,
                "titleKey": title_key,
                "rewardPoints": reward_points,
                "cooldownHours": cooldown_hours
            }),
            json_output,
            "ok kit upsert",
        ),
    }
}
