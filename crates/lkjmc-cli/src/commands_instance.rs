use serde_json::{json, Value};

use crate::commands::daemon_command;
use crate::error::CliError;

pub fn list(socket: &str, json_output: bool) -> Result<(), CliError> {
    let response = crate::client::call(socket, "instance.list", json!({}))?;
    let body = crate::format::response_body(response)?;
    if json_output {
        return crate::format::print_json(&body);
    }
    for instance in body
        .get("instances")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        println!("{}", row(instance));
    }
    Ok(())
}

pub struct CreateOptions {
    pub id: String,
    pub kind: String,
    pub template: String,
    pub command: Option<String>,
    pub jar_asset_id: Option<String>,
    pub memory_mb: Option<i64>,
    pub server_port: Option<i64>,
}

pub fn create(socket: &str, options: CreateOptions, json_output: bool) -> Result<(), CliError> {
    let mut body = json!({"id": options.id, "kind": options.kind, "template": options.template});
    if let Some(command) = options.command {
        body["command"] = Value::String(command);
    }
    if let Some(jar_asset_id) = options.jar_asset_id {
        body["jarAssetId"] = Value::String(jar_asset_id);
    }
    if let Some(memory_mb) = options.memory_mb {
        body["memoryMb"] = Value::Number(memory_mb.into());
    }
    if let Some(server_port) = options.server_port {
        body["serverPort"] = Value::Number(server_port.into());
    }
    daemon_command(
        socket,
        "instance.create",
        body,
        json_output,
        "ok instance create",
    )
}

pub fn delete(
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

fn row(instance: &Value) -> String {
    let id = text(instance, "id");
    let desired = text(instance, "desiredState");
    let observed = text(instance, "observedState");
    let port = instance
        .get("serverPort")
        .and_then(Value::as_i64)
        .map_or("-".to_string(), |v| v.to_string());
    let presence = presence(instance.get("presence"));
    format!("{id} desired={desired} observed={observed} port={port} {presence}")
}

fn presence(value: Option<&Value>) -> String {
    let Some(value) = value.and_then(Value::as_object) else {
        return "presence=unknown".to_string();
    };
    let players = value
        .get("playerCount")
        .and_then(Value::as_i64)
        .map_or("unknown".to_string(), |v| v.to_string());
    let ready = value.get("ready").and_then(Value::as_bool).unwrap_or(false);
    let reason = value
        .get("suspendReason")
        .and_then(Value::as_str)
        .unwrap_or("");
    if reason.is_empty() {
        format!("players={players} ready={ready}")
    } else {
        format!("players={players} ready={ready} suspend={reason}")
    }
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string()
}

pub fn logs(socket: &str, id: String, lines: i64, json_output: bool) -> Result<(), CliError> {
    let response = crate::client::call(socket, "instance.logs", json!({"id": id, "lines": lines}))?;
    let body = crate::format::response_body(response)?;
    if json_output {
        crate::format::print_json(&body)
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
