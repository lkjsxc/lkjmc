use serde_json::{json, Value};

use crate::commands::daemon_command;
use crate::error::CliError;

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
