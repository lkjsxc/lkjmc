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
    ConfigReload,
    DbMigrate {
        database_url: String,
    },
    DbStatus {
        database_url: String,
    },
    AuditTail {
        lines: i64,
    },
    PlayerInspect {
        player_uuid: String,
    },
    PlayerSnapshot {
        player_uuid: String,
        name: String,
        source: String,
        payload_path: String,
    },
    JarList,
    JarImport {
        kind: String,
        name: String,
        path: String,
    },
    JarInspect {
        query: String,
    },
    JarSync {
        project: String,
        channel: String,
        version: Option<String>,
    },
    JarPrune {
        yes: bool,
    },
    InstanceList,
    InstanceCreate {
        id: String,
        kind: String,
        template: String,
        command: Option<String>,
        jar_asset_id: Option<String>,
        memory_mb: Option<i64>,
        server_port: Option<i64>,
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
        [cmd, sub] if cmd == "config" && sub == "reload" => Ok(CliCommand::ConfigReload),
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
        [cmd, rest @ ..] if cmd == "jar" => crate::args_jar::parse(rest),
        [cmd, rest @ ..] if cmd == "player" => crate::args_player::parse(rest),
        [cmd, rest @ ..] if cmd == "instance" => crate::args_instance::parse(rest),
        _ => Err(CliError::message(usage())),
    }
}

pub(crate) fn value_after(values: &[String], index: usize, flag: &str) -> Result<String, CliError> {
    values
        .get(index + 1)
        .cloned()
        .ok_or_else(|| CliError::message(format!("missing value for {flag}")))
}

pub(crate) fn parse_lines(value: &str) -> Result<i64, CliError> {
    value
        .parse::<i64>()
        .map_err(|error| CliError::message(format!("invalid --lines: {error}")))
}

fn database_url() -> Result<String, CliError> {
    env::var("LKJMC_DATABASE_URL").map_err(|_| CliError::message("LKJMC_DATABASE_URL is required"))
}

fn usage() -> &'static str {
    "usage: lkjmc [--socket PATH] [--json] doctor|status|config check|config reload|db migrate|db status|audit tail|jar ...|player ...|instance ..."
}
