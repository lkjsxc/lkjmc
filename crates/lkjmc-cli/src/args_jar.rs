use crate::args::{value_after, CliCommand};
use crate::error::CliError;

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [sub] if sub == "list" => Ok(CliCommand::JarList),
        [sub, query] if sub == "inspect" => Ok(CliCommand::JarInspect {
            query: query.clone(),
        }),
        [sub, rest @ ..] if sub == "import" => parse_import(rest),
        [sub, rest @ ..] if sub == "sync" => parse_sync(rest),
        [sub, rest @ ..] if sub == "prune" => Ok(CliCommand::JarPrune {
            yes: rest.iter().any(|value| value == "--yes"),
        }),
        _ => Err(CliError::message(
            "usage: lkjmc jar list|inspect|import|sync|prune",
        )),
    }
}

fn parse_sync(values: &[String]) -> Result<CliCommand, CliError> {
    let mut project = None;
    let mut channel = Some("stable".to_string());
    let mut minecraft_release = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--project" => project = Some(value_after(values, index, "--project")?),
            "--channel" => channel = Some(value_after(values, index, "--channel")?),
            "--minecraft-release" => {
                minecraft_release = Some(value_after(values, index, "--minecraft-release")?)
            }
            other => return Err(CliError::message(format!("unknown jar sync flag: {other}"))),
        }
        index += 2;
    }
    Ok(CliCommand::JarSync {
        project: project.ok_or_else(|| CliError::message("missing --project"))?,
        channel: channel.unwrap_or_else(|| "stable".to_string()),
        minecraft_release,
    })
}

fn parse_import(values: &[String]) -> Result<CliCommand, CliError> {
    let mut kind = None;
    let mut name = None;
    let mut path = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--kind" => kind = Some(value_after(values, index, "--kind")?),
            "--name" => name = Some(value_after(values, index, "--name")?),
            "--path" => path = Some(value_after(values, index, "--path")?),
            other => {
                return Err(CliError::message(format!(
                    "unknown jar import flag: {other}"
                )))
            }
        }
        index += 2;
    }
    Ok(CliCommand::JarImport {
        kind: kind.ok_or_else(|| CliError::message("missing --kind"))?,
        name: name.ok_or_else(|| CliError::message("missing --name"))?,
        path: path.ok_or_else(|| CliError::message("missing --path"))?,
    })
}
