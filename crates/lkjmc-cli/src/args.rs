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
    ConfigCheck { path: String },
    DbMigrate { database_url: String },
    DbStatus { database_url: String },
    AuditTail { lines: i64 },
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
    let command = parse_command(&rest)?;
    Ok(CliArgs {
        socket,
        json,
        command,
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
            let lines = lines
                .parse::<i64>()
                .map_err(|error| CliError::message(format!("invalid --lines: {error}")))?;
            Ok(CliCommand::AuditTail { lines })
        }
        [cmd, sub] if cmd == "audit" && sub == "tail" => Ok(CliCommand::AuditTail { lines: 100 }),
        _ => Err(CliError::message(usage())),
    }
}

fn value_after(values: &[String], index: usize, flag: &str) -> Result<String, CliError> {
    values
        .get(index + 1)
        .cloned()
        .ok_or_else(|| CliError::message(format!("missing value for {flag}")))
}

fn database_url() -> Result<String, CliError> {
    env::var("LKJMC_DATABASE_URL").map_err(|_| CliError::message("LKJMC_DATABASE_URL is required"))
}

fn usage() -> &'static str {
    "usage: lkjmc [--socket PATH] [--json] doctor|status|config check|db migrate|db status|audit tail"
}
