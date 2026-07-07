use crate::args::CliCommand;
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityCommand {
    Plan,
    Rotate,
    Status,
    Verify,
    Create {
        surface: String,
        scopes: Vec<String>,
    },
    Revoke {
        credential_id: String,
    },
}

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [area, action] if area == "token" && action == "plan" => {
            Ok(CliCommand::Security(SecurityCommand::Plan))
        }
        [area, action] if area == "token" && action == "rotate" => {
            Ok(CliCommand::Security(SecurityCommand::Rotate))
        }
        [area, action] if area == "token" && action == "status" => {
            Ok(CliCommand::Security(SecurityCommand::Status))
        }
        [area, action] if area == "token" && action == "verify" => {
            Ok(CliCommand::Security(SecurityCommand::Verify))
        }
        [area, action, rest @ ..] if area == "token" && action == "create" => create(rest),
        [area, action, flag, id]
            if area == "token" && action == "revoke" && flag == "--credential-id" =>
        {
            Ok(CliCommand::Security(SecurityCommand::Revoke {
                credential_id: id.clone(),
            }))
        }
        _ => Err(CliError::message(usage())),
    }
}

fn create(values: &[String]) -> Result<CliCommand, CliError> {
    let mut surface = "paper".to_string();
    let mut scopes = Vec::new();
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--surface" => {
                surface = value_after(values, index, "--surface")?;
                index += 2;
            }
            "--scope" => {
                scopes.push(value_after(values, index, "--scope")?);
                index += 2;
            }
            _ => return Err(CliError::message(usage())),
        }
    }
    Ok(CliCommand::Security(SecurityCommand::Create {
        surface,
        scopes,
    }))
}

fn value_after(values: &[String], index: usize, flag: &str) -> Result<String, CliError> {
    values
        .get(index + 1)
        .cloned()
        .ok_or_else(|| CliError::message(format!("missing value for {flag}")))
}

fn usage() -> &'static str {
    "usage: lkjmc security token plan|rotate|status|verify|create [--surface NAME] [--scope SCOPE...]|revoke --credential-id ID"
}
