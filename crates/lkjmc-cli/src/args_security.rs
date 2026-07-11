use crate::args::CliCommand;
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityCommand {
    Plan, Rotate, Status, Verify,
    Create { surface: String, principal_kind: String, principal_id: String, output_file: String, expires_in_seconds: i64, scopes: Vec<String> },
    Revoke { credential_id: String },
}

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [area, action] if area == "token" && action == "plan" => Ok(CliCommand::Security(SecurityCommand::Plan)),
        [area, action] if area == "token" && action == "rotate" => Ok(CliCommand::Security(SecurityCommand::Rotate)),
        [area, action] if area == "token" && action == "status" => Ok(CliCommand::Security(SecurityCommand::Status)),
        [area, action] if area == "token" && action == "verify" => Ok(CliCommand::Security(SecurityCommand::Verify)),
        [area, action, rest @ ..] if area == "token" && action == "create" => create(rest),
        [area, action, flag, id] if area == "token" && action == "revoke" && flag == "--credential-id" => Ok(CliCommand::Security(SecurityCommand::Revoke { credential_id: id.clone() })),
        _ => Err(CliError::message(usage())),
    }
}
fn create(values: &[String]) -> Result<CliCommand, CliError> {
    let (mut surface, mut principal_kind, mut principal_id, mut output_file, mut expiry, mut scopes) = (String::new(), String::new(), String::new(), String::new(), 0, vec![]);
    let mut index = 0;
    while index < values.len() { let flag = &values[index]; let value = values.get(index + 1).cloned().ok_or_else(|| CliError::message(format!("missing value for {flag}")))?; match flag.as_str() { "--surface" => surface = value, "--principal-kind" => principal_kind = value, "--principal-id" => principal_id = value, "--output-file" => output_file = value, "--expires-in-seconds" => expiry = value.parse().map_err(|_| CliError::message("expiry must be an integer"))?, "--scope" => scopes.push(value), _ => return Err(CliError::message(usage())) }; index += 2; }
    if surface.is_empty() || principal_kind.is_empty() || principal_id.is_empty() || output_file.is_empty() || expiry <= 0 || scopes.is_empty() { return Err(CliError::message(usage())); }
    Ok(CliCommand::Security(SecurityCommand::Create { surface, principal_kind, principal_id, output_file, expires_in_seconds: expiry, scopes }))
}
fn usage() -> &'static str { "usage: lkjmc security token plan|rotate|status|verify|create --surface NAME --principal-kind KIND --principal-id ID --output-file ABSOLUTE --expires-in-seconds N --scope SCOPE...|revoke --credential-id ID" }
