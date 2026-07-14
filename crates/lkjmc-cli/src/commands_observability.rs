use serde_json::{json, Value};

use crate::args_observability::ObservabilityCommand;
use crate::error::CliError;

pub fn run(socket: &str, command: ObservabilityCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        ObservabilityCommand::Events {
            request_id,
            operation_id,
            correlation_id,
            limit,
        } => {
            let mut query = vec![format!("limit={limit}")];
            add(&mut query, "requestId", request_id);
            add(&mut query, "operationId", operation_id);
            add(&mut query, "correlationId", correlation_id);
            let value = crate::client::get(
                socket,
                &format!("/observability/events?{}", query.join("&")),
            )?;
            output(value, json_output, "ok observability events")
        }
    }
}

pub fn bundle(socket: &str, output_path: String, json_output: bool) -> Result<(), CliError> {
    let value = crate::client::post(socket, "/support/bundle", json!({"output": output_path}))?;
    output(value, json_output, "ok support bundle")
}

fn add(query: &mut Vec<String>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        query.push(format!("{key}={}", percent_encode(&value)));
    }
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
                vec![char::from(byte)]
            } else {
                format!("%{byte:02X}").chars().collect()
            }
        })
        .collect()
}

fn output(value: Value, json_output: bool, success: &str) -> Result<(), CliError> {
    if json_output {
        crate::format::print_json(&value)
    } else {
        println!("{success}");
        Ok(())
    }
}
