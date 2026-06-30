use crate::args::CliCommand;
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityCommand {
    Plan,
    Rotate,
    Status,
    Verify,
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
        _ => Err(CliError::message(
            "usage: lkjmc security token plan|rotate|status|verify",
        )),
    }
}
