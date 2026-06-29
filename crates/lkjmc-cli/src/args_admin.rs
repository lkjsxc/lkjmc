use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminCommand {
    RoleList,
}

pub fn parse(values: &[String]) -> Result<AdminCommand, CliError> {
    match values {
        [scope, action] if scope == "role" && action == "list" => Ok(AdminCommand::RoleList),
        _ => Err(CliError::message("usage: lkjmc admin role list")),
    }
}
