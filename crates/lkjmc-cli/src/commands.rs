use std::fs;
use std::process::Command;

use serde_json::{json, Value};

use crate::args::{CliArgs, CliCommand};
use crate::client;
use crate::error::CliError;
use crate::format;

pub fn run(args: CliArgs) -> Result<(), CliError> {
    match args.command {
        CliCommand::Admin(command) => crate::commands_admin::run(&args.socket, command, args.json),
        CliCommand::Announcement(command) => {
            crate::commands_announcement::run(&args.socket, command, args.json)
        }
        CliCommand::Asset(command) => crate::commands_asset::run(&args.socket, command, args.json),
        CliCommand::Bootstrap(command) => {
            crate::commands_bootstrap::run(&args.socket, command, args.json)
        }
        CliCommand::Claim(command) => crate::commands_claim::run(&args.socket, command, args.json),
        CliCommand::Doctor => {
            daemon_command(&args.socket, "doctor", json!({}), args.json, "ok doctor")
        }
        CliCommand::Status => crate::commands_status::status(&args.socket, args.json),
        CliCommand::Verify => verify(),
        CliCommand::ConfigCheck { path } => config_check(&path, args.json),
        CliCommand::ConfigReload => daemon_command(
            &args.socket,
            "config.reload",
            json!({}),
            args.json,
            "ok config reload",
        ),
        CliCommand::DbMigrate { database_url } => {
            crate::commands_db::migrate(&database_url, args.json)
        }
        CliCommand::DbStatus { database_url } => {
            crate::commands_db::status(&database_url, args.json)
        }
        CliCommand::DbResetTest { database_url } => crate::commands_db::reset_test(&database_url),
        CliCommand::AuditTail { lines } => daemon_command(
            &args.socket,
            "audit.tail",
            json!({"lines": lines}),
            args.json,
            "ok audit tail",
        ),
        CliCommand::PlayerInspect { player_uuid } => {
            crate::commands_player::inspect(&args.socket, player_uuid, args.json)
        }
        CliCommand::PlayerPointsTop { limit } => {
            crate::commands_player::points_top(&args.socket, limit, args.json)
        }
        CliCommand::PlayerSnapshot {
            player_uuid,
            name,
            source,
            payload_path,
        } => crate::commands_player::snapshot(
            &args.socket,
            player_uuid,
            name,
            source,
            payload_path,
            args.json,
        ),
        CliCommand::PlayerRestore {
            player_uuid,
            snapshot_id,
        } => crate::commands_player::restore(&args.socket, player_uuid, snapshot_id, args.json),
        CliCommand::Kit(command) => crate::commands_kit::run(&args.socket, command, args.json),
        CliCommand::Moderation(command) => {
            crate::commands_moderation::run(&args.socket, command, args.json)
        }
        CliCommand::Network(command) => crate::commands_network::run(command, args.json),
        CliCommand::Security(command) => {
            crate::commands_security::run(&args.socket, command, args.json)
        }
        CliCommand::Shop(command) => crate::commands_shop::run(&args.socket, command, args.json),
        CliCommand::Vote(command) => crate::commands_vote::run(&args.socket, command, args.json),
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
            minecraft_release,
        } => {
            crate::commands_jar::sync(&args.socket, project, channel, minecraft_release, args.json)
        }
        CliCommand::JarPrune { yes } => crate::commands_jar::prune(&args.socket, yes, args.json),
        CliCommand::InstanceList => crate::commands_instance::list(&args.socket, args.json),
        CliCommand::InstanceCreate {
            id,
            kind,
            template,
            command,
            jar_asset_id,
            memory_mb,
            server_port,
            accept_minecraft_eula,
        } => crate::commands_instance::create(
            &args.socket,
            crate::commands_instance::CreateOptions {
                id,
                kind,
                template,
                command,
                jar_asset_id,
                memory_mb,
                server_port,
                accept_minecraft_eula,
            },
            args.json,
        ),
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
            crate::commands_instance::delete(&args.socket, id, yes, force, args.json)
        }
        CliCommand::InstanceLogs { id, lines } => {
            crate::commands_instance::logs(&args.socket, id, lines, args.json)
        }
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

fn verify() -> Result<(), CliError> {
    let status = Command::new("./scripts/verify.sh").status()?;
    if status.success() {
        return Ok(());
    }
    Err(CliError::message(format!(
        "verify failed with status {status}"
    )))
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
