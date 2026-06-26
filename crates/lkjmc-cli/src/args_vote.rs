use crate::args::{parse_lines, value_after};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteCommand {
    List,
    UpsertLink {
        id: String,
        title_key: String,
        url: String,
    },
    Reward {
        player_uuid: String,
        player_name: String,
        link_id: String,
        points: i64,
    },
}

pub fn parse(values: &[String]) -> Result<VoteCommand, CliError> {
    match values {
        [sub] if sub == "list" => Ok(VoteCommand::List),
        [sub, action, id, rest @ ..] if sub == "link" && action == "upsert" => upsert(id, rest),
        [sub, player_uuid, player_name, rest @ ..] if sub == "reward" => {
            reward(player_uuid, player_name, rest)
        }
        _ => Err(CliError::message(usage())),
    }
}

fn upsert(id: &str, values: &[String]) -> Result<VoteCommand, CliError> {
    if values.len() != 4 {
        return Err(CliError::message(usage()));
    }
    let mut title_key = String::new();
    let mut url = String::new();
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--title-key" => title_key = value_after(values, index, "--title-key")?,
            "--url" => url = value_after(values, index, "--url")?,
            _ => return Err(CliError::message(usage())),
        }
        index += 2;
    }
    Ok(VoteCommand::UpsertLink {
        id: id.to_string(),
        title_key,
        url,
    })
}

fn reward(
    player_uuid: &str,
    player_name: &str,
    values: &[String],
) -> Result<VoteCommand, CliError> {
    if values.len() != 4 {
        return Err(CliError::message(usage()));
    }
    let mut link_id = String::new();
    let mut points = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--link" => link_id = value_after(values, index, "--link")?,
            "--points" => points = Some(parse_lines(&value_after(values, index, "--points")?)?),
            _ => return Err(CliError::message(usage())),
        }
        index += 2;
    }
    Ok(VoteCommand::Reward {
        player_uuid: player_uuid.to_string(),
        player_name: player_name.to_string(),
        link_id,
        points: points.ok_or_else(|| CliError::message(usage()))?,
    })
}

fn usage() -> &'static str {
    "usage: lkjmc vote list | vote link upsert ID --title-key KEY --url URL | vote reward UUID NAME --link ID --points POINTS"
}
