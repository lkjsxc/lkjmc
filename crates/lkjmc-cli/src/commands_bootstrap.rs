use serde_json::json;

use crate::args_bootstrap::BootstrapCommand;
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: BootstrapCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        BootstrapCommand::Plan { profile, bedrock } => {
            let mut body = json!({"profile": profile});
            if let Some(bedrock) = bedrock {
                body["bedrock"] = json!(bedrock);
            }
            daemon_command(
                socket,
                "bootstrap.plan",
                body,
                json_output,
                "ok bootstrap plan",
            )
        }
        BootstrapCommand::Apply {
            profile,
            accept_minecraft_eula,
            bedrock,
        } => {
            let mut body = json!({
                "profile": profile,
                "acceptMinecraftEula": accept_minecraft_eula
            });
            if let Some(bedrock) = bedrock {
                body["bedrock"] = json!(bedrock);
            }
            daemon_command(
                socket,
                "bootstrap.apply",
                body,
                json_output,
                "ok bootstrap apply",
            )
        }
        BootstrapCommand::Status => daemon_command(
            socket,
            "bootstrap.status",
            json!({}),
            json_output,
            "ok bootstrap status",
        ),
        BootstrapCommand::Doctor => daemon_command(
            socket,
            "bootstrap.doctor",
            json!({}),
            json_output,
            "ok bootstrap doctor",
        ),
    }
}
