use crate::args::{parse_lines, value_after};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModerationCommand {
    Reports {
        limit: i64,
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
    "usage: lkjmc moderation reports [--limit N] | moderation ban UUID NAME --reason REASON | moderation unban NAME | moderation status UUID"
}
