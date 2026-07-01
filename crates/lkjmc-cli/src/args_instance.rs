use crate::args::{parse_lines, value_after, CliCommand};
use crate::error::CliError;

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [sub] if sub == "list" => Ok(CliCommand::InstanceList),
        [sub, rest @ ..] if sub == "create" => parse_create(rest),
        [sub, id] if sub == "start" => Ok(CliCommand::InstanceStart { id: id.clone() }),
        [sub, id] if sub == "stop" => Ok(CliCommand::InstanceStop { id: id.clone() }),
        [sub, id] if sub == "restart" => Ok(CliCommand::InstanceRestart { id: id.clone() }),
        [sub, rest @ ..] if sub == "delete" => parse_delete(rest),
        [sub, rest @ ..] if sub == "logs" => parse_logs(rest),
        _ => Err(CliError::message("usage: lkjmc instance ...")),
    }
}

fn parse_create(values: &[String]) -> Result<CliCommand, CliError> {
    let mut id = None;
    let mut kind = None;
    let mut template = None;
    let mut command = None;
    let mut jar_asset_id = None;
    let mut memory_mb = None;
    let mut server_port = None;
    let mut accept_minecraft_eula = false;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--id" => id = Some(value_after(values, index, "--id")?),
            "--kind" => kind = Some(value_after(values, index, "--kind")?),
            "--template" => template = Some(value_after(values, index, "--template")?),
            "--command" => command = Some(value_after(values, index, "--command")?),
            "--jar-asset" => jar_asset_id = Some(value_after(values, index, "--jar-asset")?),
            "--memory-mb" => {
                memory_mb = Some(parse_memory(&value_after(values, index, "--memory-mb")?)?)
            }
            "--server-port" => {
                server_port = Some(parse_port(&value_after(values, index, "--server-port")?)?)
            }
            "--accept-minecraft-eula" => {
                accept_minecraft_eula = true;
                index += 1;
                continue;
            }
            other => return Err(CliError::message(format!("unknown create flag: {other}"))),
        }
        index += 2;
    }
    Ok(CliCommand::InstanceCreate {
        id: id.ok_or_else(|| CliError::message("missing --id"))?,
        kind: kind.ok_or_else(|| CliError::message("missing --kind"))?,
        template: template.ok_or_else(|| CliError::message("missing --template"))?,
        command,
        jar_asset_id,
        memory_mb,
        server_port,
        accept_minecraft_eula,
    })
}

fn parse_port(value: &str) -> Result<i64, CliError> {
    value
        .parse::<i64>()
        .map_err(|error| CliError::message(format!("invalid --server-port: {error}")))
}

fn parse_memory(value: &str) -> Result<i64, CliError> {
    value
        .parse::<i64>()
        .map_err(|error| CliError::message(format!("invalid --memory-mb: {error}")))
}

fn parse_delete(values: &[String]) -> Result<CliCommand, CliError> {
    let id = values
        .first()
        .cloned()
        .ok_or_else(|| CliError::message("missing instance id"))?;
    let yes = values.iter().any(|value| value == "--yes");
    let force = values.iter().any(|value| value == "--force");
    Ok(CliCommand::InstanceDelete { id, yes, force })
}

fn parse_logs(values: &[String]) -> Result<CliCommand, CliError> {
    let id = values
        .first()
        .cloned()
        .ok_or_else(|| CliError::message("missing instance id"))?;
    let mut lines = 120;
    if values.len() == 3 && values[1] == "--lines" {
        lines = parse_lines(&values[2])?;
    }
    Ok(CliCommand::InstanceLogs { id, lines })
}
