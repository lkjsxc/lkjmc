use std::fs;

use serde_json::json;

use crate::args::{CliArgs, CliCommand};
use crate::client;
use crate::error::CliError;
use crate::format;

pub fn run(args: CliArgs) -> Result<(), CliError> {
    match args.command {
        CliCommand::Doctor => {
            daemon_command(&args.socket, "doctor", json!({}), args.json, "ok doctor")
        }
        CliCommand::Status => daemon_command(
            &args.socket,
            "status",
            json!({}),
            args.json,
            "daemon running",
        ),
        CliCommand::ConfigCheck { path } => config_check(&path, args.json),
        CliCommand::DbMigrate { database_url } => db_migrate(&database_url, args.json),
        CliCommand::DbStatus { database_url } => db_status(&database_url, args.json),
        CliCommand::AuditTail { lines } => daemon_command(
            &args.socket,
            "audit.tail",
            json!({"lines": lines}),
            args.json,
            "ok audit tail",
        ),
    }
}

fn daemon_command(
    socket: &str,
    command: &str,
    body: serde_json::Value,
    json_output: bool,
    human: &str,
) -> Result<(), CliError> {
    let response = client::call(socket, command, body)?;
    let body = format::response_body(response)?;
    if json_output {
        format::print_json(&body)
    } else {
        println!("{human}");
        Ok(())
    }
}

fn config_check(path: &str, json_output: bool) -> Result<(), CliError> {
    let content = fs::read_to_string(path)?;
    let config = lkjmc_core::config::LkjmcConfig::from_json_str(&content)?;
    if json_output {
        format::print_json(&json!({"ok": true, "installRoot": config.install_root}))
    } else {
        println!("ok config check");
        Ok(())
    }
}

fn db_migrate(database_url: &str, json_output: bool) -> Result<(), CliError> {
    let mut client = lkjmc_store::pool::connect(database_url)?;
    let applied = lkjmc_store::migrate::apply(&mut client)?;
    if json_output {
        format::print_json(&json!({"applied": applied}))
    } else {
        println!("ok db migrate {}", applied.len());
        Ok(())
    }
}

fn db_status(database_url: &str, json_output: bool) -> Result<(), CliError> {
    let mut client = lkjmc_store::pool::connect(database_url)?;
    let versions = lkjmc_store::migrate::applied_versions(&mut client)?;
    if json_output {
        format::print_json(&json!({"versions": versions}))
    } else {
        println!("ok db status {}", versions.len());
        Ok(())
    }
}
