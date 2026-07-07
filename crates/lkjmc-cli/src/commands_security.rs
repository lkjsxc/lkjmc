use serde_json::{json, Value};

use crate::args_security::SecurityCommand;
use crate::{client, error::CliError, format};

pub fn run(socket: &str, command: SecurityCommand, json_output: bool) -> Result<(), CliError> {
    let (daemon, body) = match &command {
        SecurityCommand::Plan => ("security.daemon-token.plan", json!({})),
        SecurityCommand::Rotate => ("security.daemon-token.rotate", json!({})),
        SecurityCommand::Status => ("security.daemon-token.status", json!({})),
        SecurityCommand::Verify => ("security.daemon-token.verify", json!({})),
        SecurityCommand::Create { surface, scopes } => (
            "security.daemon-token.create",
            json!({"surface": surface, "scopes": scopes}),
        ),
        SecurityCommand::Revoke { credential_id } => (
            "security.daemon-token.revoke",
            json!({"credentialId": credential_id}),
        ),
    };
    let body = format::response_body(client::call(socket, daemon, body)?)?;
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
            "token status: configured={} fingerprint={} scoped={}",
            body.get("configured")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            str_field(body, "fingerprint").unwrap_or("none"),
            body.get("scopedTokenCount")
                .and_then(Value::as_i64)
                .unwrap_or(0)
        ),
        SecurityCommand::Verify => format!(
            "token verify: configured={}",
            body.get("configured")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        ),
        SecurityCommand::Create { .. } => format!(
            "created scoped token: credential={} fingerprint={} token={}",
            str_field(body, "credentialId").unwrap_or("unknown"),
            str_field(body, "fingerprint").unwrap_or("redacted"),
            str_field(body, "token").unwrap_or("not-returned")
        ),
        SecurityCommand::Revoke { .. } => format!(
            "revoked scoped token: credential={} revoked={}",
            str_field(body, "credentialId").unwrap_or("unknown"),
            body.get("revoked")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        ),
    }
}

fn str_field<'a>(body: &'a Value, key: &str) -> Option<&'a str> {
    body.get(key).and_then(Value::as_str)
}
