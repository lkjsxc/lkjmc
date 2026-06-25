use crate::args::{parse_lines, value_after};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KitCommand {
    List,
    Upsert {
        id: String,
        title_key: String,
        reward_points: i64,
        cooldown_hours: i64,
    },
}

pub fn parse(values: &[String]) -> Result<KitCommand, CliError> {
    match values {
        [sub] if sub == "list" => Ok(KitCommand::List),
        [sub, id, rest @ ..] if sub == "upsert" => upsert(id, rest),
        _ => Err(CliError::message(usage())),
    }
}

fn upsert(id: &str, values: &[String]) -> Result<KitCommand, CliError> {
    if values.len() != 6 {
        return Err(CliError::message(usage()));
    }
    let mut title_key = String::new();
    let mut reward_points = None;
    let mut cooldown_hours = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--title-key" => title_key = value_after(values, index, "--title-key")?,
            "--reward-points" => {
                reward_points = Some(parse_lines(&value_after(
                    values,
                    index,
                    "--reward-points",
                )?)?)
            }
            "--cooldown-hours" => {
                cooldown_hours = Some(parse_lines(&value_after(
                    values,
                    index,
                    "--cooldown-hours",
                )?)?)
            }
            _ => return Err(CliError::message(usage())),
        }
        index += 2;
    }
    Ok(KitCommand::Upsert {
        id: id.to_string(),
        title_key,
        reward_points: reward_points.ok_or_else(|| CliError::message(usage()))?,
        cooldown_hours: cooldown_hours.ok_or_else(|| CliError::message(usage()))?,
    })
}

fn usage() -> &'static str {
    "usage: lkjmc kit list | kit upsert KIT --title-key KEY --reward-points POINTS --cooldown-hours HOURS"
}
