use std::env;

use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub socket: String,
    pub json: bool,
    pub command: CliCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Doctor,
    Status,
    ConfigCheck {
        path: String,
    },
    DbMigrate {
        database_url: String,
    },
    DbStatus {
        database_url: String,
    },
    AuditTail {
        lines: i64,
    },
    InstanceList,
    InstanceCreate {
        id: String,
        kind: String,
        template: String,
        command: Option<String>,
    },
    InstanceStart {
        id: String,
    },
    InstanceStop {
        id: String,
    },
    InstanceRestart {
        id: String,
    },
    InstanceDelete {
        id: String,
        yes: bool,
        force: bool,
    },
    InstanceLogs {
        id: String,
        lines: i64,
    },
}

pub fn parse(values: Vec<String>) -> Result<CliArgs, CliError> {
    let mut socket = "/run/lkjmc/daemon.sock".to_string();
    let mut json = false;
    let mut rest = Vec::new();
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--socket" => {
                socket = value_after(&values, index, "--socket")?;
                index += 2;
            }
            "--json" => {
                json = true;
                index += 1;
            }
            value => {
                rest.push(value.to_string());
                index += 1;
            }
        }
    }
    Ok(CliArgs {
        socket,
        json,
        command: parse_command(&rest)?,
    })
}

fn parse_command(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [cmd] if cmd == "doctor" => Ok(CliCommand::Doctor),
        [cmd] if cmd == "status" => Ok(CliCommand::Status),
        [cmd, sub] if cmd == "config" && sub == "check" => Ok(CliCommand::ConfigCheck {
            path: "/etc/lkjmc/lkjmc.json".to_string(),
        }),
        [cmd, sub, flag, path] if cmd == "config" && sub == "check" && flag == "--path" => {
            Ok(CliCommand::ConfigCheck { path: path.clone() })
        }
        [cmd, sub] if cmd == "db" && sub == "migrate" => Ok(CliCommand::DbMigrate {
            database_url: database_url()?,
        }),
        [cmd, sub] if cmd == "db" && sub == "status" => Ok(CliCommand::DbStatus {
            database_url: database_url()?,
        }),
        [cmd, sub, flag, lines] if cmd == "audit" && sub == "tail" && flag == "--lines" => {
            Ok(CliCommand::AuditTail {
                lines: parse_lines(lines)?,
            })
        }
        [cmd, sub] if cmd == "audit" && sub == "tail" => Ok(CliCommand::AuditTail { lines: 100 }),
        [cmd, sub] if cmd == "instance" && sub == "list" => Ok(CliCommand::InstanceList),
        [cmd, sub, rest @ ..] if cmd == "instance" && sub == "create" => {
            parse_instance_create(rest)
        }
        [cmd, sub, id] if cmd == "instance" && sub == "start" => {
            Ok(CliCommand::InstanceStart { id: id.clone() })
        }
        [cmd, sub, id] if cmd == "instance" && sub == "stop" => {
            Ok(CliCommand::InstanceStop { id: id.clone() })
        }
        [cmd, sub, id] if cmd == "instance" && sub == "restart" => {
            Ok(CliCommand::InstanceRestart { id: id.clone() })
        }
        [cmd, sub, rest @ ..] if cmd == "instance" && sub == "delete" => {
            parse_instance_delete(rest)
        }
        [cmd, sub, rest @ ..] if cmd == "instance" && sub == "logs" => parse_instance_logs(rest),
        _ => Err(CliError::message(usage())),
    }
}

fn parse_instance_create(values: &[String]) -> Result<CliCommand, CliError> {
    let mut id = None;
    let mut kind = None;
    let mut template = None;
    let mut command = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--id" => id = Some(value_after(values, index, "--id")?),
            "--kind" => kind = Some(value_after(values, index, "--kind")?),
            "--template" => template = Some(value_after(values, index, "--template")?),
            "--command" => command = Some(value_after(values, index, "--command")?),
            other => return Err(CliError::message(format!("unknown create flag: {other}"))),
        }
        index += 2;
    }
    Ok(CliCommand::InstanceCreate {
        id: id.ok_or_else(|| CliError::message("missing --id"))?,
        kind: kind.ok_or_else(|| CliError::message("missing --kind"))?,
        template: template.ok_or_else(|| CliError::message("missing --template"))?,
        command,
    })
}

fn parse_instance_delete(values: &[String]) -> Result<CliCommand, CliError> {
    let id = values
        .first()
        .cloned()
        .ok_or_else(|| CliError::message("missing instance id"))?;
    let yes = values.iter().any(|value| value == "--yes");
    let force = values.iter().any(|value| value == "--force");
    Ok(CliCommand::InstanceDelete { id, yes, force })
}

fn parse_instance_logs(values: &[String]) -> Result<CliCommand, CliError> {
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

fn value_after(values: &[String], index: usize, flag: &str) -> Result<String, CliError> {
    values
        .get(index + 1)
        .cloned()
        .ok_or_else(|| CliError::message(format!("missing value for {flag}")))
}

fn parse_lines(value: &str) -> Result<i64, CliError> {
    value
        .parse::<i64>()
        .map_err(|error| CliError::message(format!("invalid --lines: {error}")))
}

fn database_url() -> Result<String, CliError> {
    env::var("LKJMC_DATABASE_URL").map_err(|_| CliError::message("LKJMC_DATABASE_URL is required"))
}

fn usage() -> &'static str {
    "usage: lkjmc [--socket PATH] [--json] doctor|status|config check|db migrate|db status|audit tail|instance ..."
}
