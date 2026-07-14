use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use serde_json::Value;
use uuid::Uuid;

use crate::error::CliError;

pub fn get(socket: &str, path: &str) -> Result<Value, CliError> {
    request(socket, reqwest::Method::GET, path, None)
}

pub fn post(socket: &str, path: &str, body: Value) -> Result<Value, CliError> {
    request(socket, reqwest::Method::POST, path, Some(body))
}

fn request(
    socket: &str,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
) -> Result<Value, CliError> {
    let client = reqwest::blocking::Client::builder()
        .unix_socket(socket)
        .build()
        .map_err(|error| CliError::message(error.to_string()))?;
    let mut request = client.request(method, format!("http://localhost{path}"));
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request
        .send()
        .map_err(|error| CliError::message(error.to_string()))?;
    if !response.status().is_success() {
        return Err(CliError::message(format!(
            "daemon returned HTTP {}",
            response.status()
        )));
    }
    response
        .json::<Value>()
        .map_err(|error| CliError::message(error.to_string()))
}

pub fn call(socket: &str, command: &str, body: Value) -> Result<CommandResponse, CliError> {
    lkjmc_core::command_registry::validate_body(command, &body)
        .map_err(|error| CliError::message(format!("invalid {command} body: {error}")))?;
    let request = CommandEnvelope {
        request_id: CommandId::parse("request id", Uuid::new_v4().to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "local-shell".to_string(),
        },
        command: command.to_string(),
        body,
    };
    let client = reqwest::blocking::Client::builder()
        .unix_socket(socket)
        .build()
        .map_err(|error| CliError::message(error.to_string()))?;
    let response = client
        .post("http://localhost/command")
        .json(&request)
        .send()
        .map_err(|error| CliError::message(error.to_string()))?;
    if !response.status().is_success() {
        return Err(CliError::message(format!(
            "daemon returned HTTP {}",
            response.status()
        )));
    }
    response
        .json::<CommandResponse>()
        .map_err(|error| CliError::message(error.to_string()))
}
