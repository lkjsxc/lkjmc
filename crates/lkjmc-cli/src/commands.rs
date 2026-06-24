use std::fs;

use serde_json::{json, Value};

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
        CliCommand::JarList => crate::commands_jar::list(&args.socket, args.json),
        CliCommand::JarImport { kind, name, path } => {
            crate::commands_jar::import(&args.socket, kind, name, path, args.json)
        }
        CliCommand::JarInspect { query } => {
            crate::commands_jar::inspect(&args.socket, query, args.json)
        }
        CliCommand::JarSync {
            project,
            channel,
            version,
        } => crate::commands_jar::sync(&args.socket, project, channel, version, args.json),
        CliCommand::InstanceList => daemon_command(
            &args.socket,
            "instance.list",
            json!({}),
            args.json,
            "ok instance list",
        ),
        CliCommand::InstanceCreate {
            id,
            kind,
            template,
            command,
            jar_asset_id,
            memory_mb,
            server_port,
        } => {
            let mut body = json!({"id": id, "kind": kind, "template": template});
            if let Some(command) = command {
                body["command"] = Value::String(command);
            }
            if let Some(jar_asset_id) = jar_asset_id {
                body["jarAssetId"] = Value::String(jar_asset_id);
            }
            if let Some(memory_mb) = memory_mb {
                body["memoryMb"] = Value::Number(memory_mb.into());
            }
            if let Some(server_port) = server_port {
                body["serverPort"] = Value::Number(server_port.into());
            }
            daemon_command(
                &args.socket,
                "instance.create",
                body,
                args.json,
                "ok instance create",
            )
        }
        CliCommand::InstanceStart { id } => daemon_command(
            &args.socket,
            "instance.start",
            json!({"id": id}),
            args.json,
            "ok instance start",
        ),
        CliCommand::InstanceStop { id } => daemon_command(
            &args.socket,
            "instance.stop",
            json!({"id": id}),
            args.json,
            "ok instance stop",
        ),
        CliCommand::InstanceRestart { id } => daemon_command(
            &args.socket,
            "instance.restart",
            json!({"id": id}),
            args.json,
            "ok instance restart",
        ),
        CliCommand::InstanceDelete { id, yes, force } => {
            instance_delete(&args.socket, id, yes, force, args.json)
        }
        CliCommand::InstanceLogs { id, lines } => instance_logs(&args.socket, id, lines, args.json),
    }
}

pub(crate) fn daemon_command(
    socket: &str,
    command: &str,
    body: Value,
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

fn instance_delete(
    socket: &str,
    id: String,
    yes: bool,
    force: bool,
    json_output: bool,
) -> Result<(), CliError> {
    if !yes {
        return Err(CliError::message("instance delete requires --yes"));
    }
    daemon_command(
        socket,
        "instance.delete",
        json!({"id": id, "force": force}),
        json_output,
        "ok instance delete",
    )
}

fn instance_logs(socket: &str, id: String, lines: i64, json_output: bool) -> Result<(), CliError> {
    let response = client::call(socket, "instance.logs", json!({"id": id, "lines": lines}))?;
    let body = format::response_body(response)?;
    if json_output {
        format::print_json(&body)
    } else {
        for line in body
            .get("lines")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(value) = line.as_str() {
                println!("{value}");
            }
        }
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
