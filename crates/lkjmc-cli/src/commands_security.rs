use serde_json::{json, Value};

use crate::args_security::SecurityCommand;
use crate::{client, error::CliError, format};

pub fn run(socket: &str, command: SecurityCommand, json_output: bool) -> Result<(), CliError> {
    let daemon = match command {
        SecurityCommand::Plan => "security.daemon-token.plan",
        SecurityCommand::Rotate => "security.daemon-token.rotate",
        SecurityCommand::Status => "security.daemon-token.status",
        SecurityCommand::Verify => "security.daemon-token.verify",
    };
    let body = format::response_body(client::call(socket, daemon, json!({}))?)?;
    if json_output {
        return format::print_json(&body);
    }
    println!("{}", human(command, &body));
    Ok(())
}

fn human(command: SecurityCommand, body: &Value) -> String {
    match command {
        SecurityCommand::Plan => format!(
            "token rotation plan: file={} consumer={}",
            str_field(body, "tokenFile").unwrap_or("not-configured"),
            str_field(body, "consumerAction").unwrap_or("unknown")
        ),
        SecurityCommand::Rotate => format!(
            "ok token rotate: file={} fingerprint={}",
            str_field(body, "tokenFile").unwrap_or("not-configured"),
            str_field(body, "fingerprint").unwrap_or("redacted")
        ),
        SecurityCommand::Status => format!(
            "token status: configured={} fingerprint={}",
            body.get("configured")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            str_field(body, "fingerprint").unwrap_or("none")
        ),
        SecurityCommand::Verify => format!(
            "token verify: configured={}",
            body.get("configured")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        ),
    }
}

fn str_field<'a>(body: &'a Value, key: &str) -> Option<&'a str> {
    body.get(key).and_then(Value::as_str)
}
