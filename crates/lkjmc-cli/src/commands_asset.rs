use serde_json::json;

use crate::args_asset::AssetCommand;
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: AssetCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        AssetCommand::ServerSync {
            project,
            minecraft_release,
        } => {
            let mut body = json!({"project": project});
            if let Some(release) = minecraft_release {
                body["minecraftRelease"] = json!(release);
            }
            daemon_command(
                socket,
                "asset.server.sync",
                body,
                json_output,
                "ok asset server sync",
            )
        }
        AssetCommand::PluginSync { plugin } => daemon_command(
            socket,
            "asset.plugin.sync",
            json!({"plugin": plugin}),
            json_output,
            "ok asset plugin sync",
        ),
        AssetCommand::PluginList => daemon_command(
            socket,
            "asset.plugin.list",
            json!({}),
            json_output,
            "ok asset plugin list",
        ),
        AssetCommand::PluginInspect { plugin } => daemon_command(
            socket,
            "asset.plugin.inspect",
            json!({"plugin": plugin}),
            json_output,
            "ok asset plugin inspect",
        ),
    }
}
