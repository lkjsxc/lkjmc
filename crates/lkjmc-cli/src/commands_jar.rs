use serde_json::json;

use crate::commands::daemon_command;
use crate::error::CliError;

pub fn list(socket: &str, json_output: bool) -> Result<(), CliError> {
    daemon_command(socket, "jar.list", json!({}), json_output, "ok jar list")
}

pub fn import(
    socket: &str,
    kind: String,
    name: String,
    path: String,
    json_output: bool,
) -> Result<(), CliError> {
    daemon_command(
        socket,
        "jar.import",
        json!({"kind": kind, "name": name, "path": path}),
        json_output,
        "ok jar import",
    )
}

pub fn inspect(socket: &str, query: String, json_output: bool) -> Result<(), CliError> {
    daemon_command(
        socket,
        "jar.inspect",
        json!({"query": query}),
        json_output,
        "ok jar inspect",
    )
}

pub fn sync(
    socket: &str,
    project: String,
    channel: String,
    version: Option<String>,
    json_output: bool,
) -> Result<(), CliError> {
    let mut body = json!({"project": project, "channel": channel});
    if let Some(version) = version {
        body["version"] = serde_json::Value::String(version);
    }
    daemon_command(socket, "jar.sync", body, json_output, "ok jar sync")
}
