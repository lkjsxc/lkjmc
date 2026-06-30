use crate::args::CliCommand;
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityCommand {
    TokenPlan,
    TokenRotate,
    TokenStatus,
    TokenVerify,
}

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [area, action] if area == "token" && action == "plan" => {
            Ok(CliCommand::Security(SecurityCommand::TokenPlan))
        }
        [area, action] if area == "token" && action == "rotate" => {
            Ok(CliCommand::Security(SecurityCommand::TokenRotate))
        }
        [area, action] if area == "token" && action == "status" => {
            Ok(CliCommand::Security(SecurityCommand::TokenStatus))
        }
        [area, action] if area == "token" && action == "verify" => {
            Ok(CliCommand::Security(SecurityCommand::TokenVerify))
        }
        _ => Err(CliError::message(
            "usage: lkjmc security token plan|rotate|status|verify",
        )),
    }
}
