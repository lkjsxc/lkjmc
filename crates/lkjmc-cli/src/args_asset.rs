use crate::args::{value_after, CliCommand};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetCommand {
    ServerSync {
        project: String,
        minecraft_release: Option<String>,
    },
    PluginSync {
        plugin: String,
    },
    PluginList,
    PluginInspect {
        plugin: String,
    },
}

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [family, action, rest @ ..] if family == "server" && action == "sync" => {
            Ok(CliCommand::Asset(parse_server_sync(rest)?))
        }
        [family, action, rest @ ..] if family == "plugin" && action == "sync" => {
            Ok(CliCommand::Asset(parse_plugin_sync(rest)?))
        }
        [family, action] if family == "plugin" && action == "list" => {
            Ok(CliCommand::Asset(AssetCommand::PluginList))
        }
        [family, action, plugin] if family == "plugin" && action == "inspect" => {
            Ok(CliCommand::Asset(AssetCommand::PluginInspect {
                plugin: plugin.clone(),
            }))
        }
        _ => Err(CliError::message(
            "usage: lkjmc asset server sync|plugin sync|plugin list|plugin inspect",
        )),
    }
}

fn parse_server_sync(values: &[String]) -> Result<AssetCommand, CliError> {
    let mut project = None;
    let mut minecraft_release = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--project" => project = Some(value_after(values, index, "--project")?),
            "--minecraft-release" => {
                minecraft_release = Some(value_after(values, index, "--minecraft-release")?)
            }
            other => return Err(CliError::message(format!("unknown asset flag: {other}"))),
        }
        index += 2;
    }
    Ok(AssetCommand::ServerSync {
        project: project.ok_or_else(|| CliError::message("missing --project"))?,
        minecraft_release,
    })
}

fn parse_plugin_sync(values: &[String]) -> Result<AssetCommand, CliError> {
    let mut plugin = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--plugin" => plugin = Some(value_after(values, index, "--plugin")?),
            other => return Err(CliError::message(format!("unknown asset flag: {other}"))),
        }
        index += 2;
    }
    Ok(AssetCommand::PluginSync {
        plugin: plugin.ok_or_else(|| CliError::message("missing --plugin"))?,
    })
}
