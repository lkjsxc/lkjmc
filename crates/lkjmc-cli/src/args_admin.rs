use crate::args::{parse_lines, value_after};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminCommand {
    RoleList,
    Grant {
        principal: String,
        role: String,
        reason: String,
    },
    Revoke {
        principal: String,
        role: String,
        reason: String,
    },
    Inspect {
        principal: String,
    },
    Audit {
        lines: i64,
    },
}

pub fn parse(values: &[String]) -> Result<AdminCommand, CliError> {
    match values {
        [scope, action] if scope == "role" && action == "list" => Ok(AdminCommand::RoleList),
        [cmd, principal, role, flag, reason] if cmd == "grant" && flag == "--reason" => {
            Ok(AdminCommand::Grant {
                principal: principal.clone(),
                role: role.clone(),
                reason: reason.clone(),
            })
        }
        [cmd, principal, role, flag, reason] if cmd == "revoke" && flag == "--reason" => {
            Ok(AdminCommand::Revoke {
                principal: principal.clone(),
                role: role.clone(),
                reason: reason.clone(),
            })
        }
        [cmd, principal] if cmd == "inspect" => Ok(AdminCommand::Inspect {
            principal: principal.clone(),
        }),
        [cmd] if cmd == "audit" => Ok(AdminCommand::Audit { lines: 50 }),
        [cmd, flag, lines] if cmd == "audit" && flag == "--lines" => Ok(AdminCommand::Audit {
            lines: parse_lines(lines)?,
        }),
        [cmd, rest @ ..] if cmd == "audit" => audit(rest),
        _ => Err(CliError::message(usage())),
    }
}

fn audit(values: &[String]) -> Result<AdminCommand, CliError> {
    if values.len() == 2 && values[0] == "--lines" {
        return Ok(AdminCommand::Audit {
            lines: parse_lines(&value_after(values, 0, "--lines")?)?,
        });
    }
    Err(CliError::message(usage()))
}

pub fn principal(value: &str) -> (String, String) {
    value
        .split_once(':')
        .map(|(kind, id)| (kind.to_string(), id.to_string()))
        .unwrap_or_else(|| ("minecraft-player".to_string(), value.to_string()))
}

fn usage() -> &'static str {
    "usage: lkjmc admin role list | admin grant PRINCIPAL ROLE --reason TEXT | admin revoke PRINCIPAL ROLE --reason TEXT | admin inspect PRINCIPAL | admin audit [--lines N]"
}
