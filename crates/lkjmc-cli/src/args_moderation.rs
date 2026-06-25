use crate::args::{parse_lines, value_after};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModerationCommand {
    Reports {
        limit: i64,
    },
    ReportClose {
        report_id: String,
        status: String,
    },
    Ban {
        player_uuid: String,
        player_name: String,
        reason: String,
    },
    Unban {
        player_name: String,
    },
    Status {
        player_uuid: String,
    },
}

pub fn parse(values: &[String]) -> Result<ModerationCommand, CliError> {
    match values {
        [sub] if sub == "reports" => Ok(ModerationCommand::Reports { limit: 20 }),
        [sub, flag, limit] if sub == "reports" && flag == "--limit" => {
            Ok(ModerationCommand::Reports {
                limit: parse_lines(limit)?,
            })
        }
        [sub, action, report_id] if sub == "report" => report_close(action, report_id),
        [sub, player_name] if sub == "unban" => Ok(ModerationCommand::Unban {
            player_name: player_name.clone(),
        }),
        [sub, player_uuid] if sub == "status" => Ok(ModerationCommand::Status {
            player_uuid: player_uuid.clone(),
        }),
        [sub, player_uuid, player_name, rest @ ..] if sub == "ban" => {
            ban(player_uuid, player_name, rest)
        }
        _ => Err(CliError::message(usage())),
    }
}

fn report_close(action: &str, report_id: &str) -> Result<ModerationCommand, CliError> {
    match action {
        "resolve" | "dismiss" => Ok(ModerationCommand::ReportClose {
            report_id: report_id.to_string(),
            status: action.to_string(),
        }),
        _ => Err(CliError::message(usage())),
    }
}

fn ban(
    player_uuid: &str,
    player_name: &str,
    values: &[String],
) -> Result<ModerationCommand, CliError> {
    if values.len() != 2 || values[0] != "--reason" {
        return Err(CliError::message(usage()));
    }
    Ok(ModerationCommand::Ban {
        player_uuid: player_uuid.to_string(),
        player_name: player_name.to_string(),
        reason: value_after(values, 0, "--reason")?,
    })
}

fn usage() -> &'static str {
    "usage: lkjmc moderation reports [--limit N] | moderation report resolve|dismiss ID | moderation ban UUID NAME --reason REASON | moderation unban NAME | moderation status UUID"
}
