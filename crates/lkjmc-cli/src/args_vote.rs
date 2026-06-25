use crate::args::value_after;
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteCommand {
    List,
    UpsertLink {
        id: String,
        title_key: String,
        url: String,
    },
}

pub fn parse(values: &[String]) -> Result<VoteCommand, CliError> {
    match values {
        [sub] if sub == "list" => Ok(VoteCommand::List),
        [sub, action, id, rest @ ..] if sub == "link" && action == "upsert" => upsert(id, rest),
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

fn usage() -> &'static str {
    "usage: lkjmc vote list | vote link upsert ID --title-key KEY --url URL"
}
