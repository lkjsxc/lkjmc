use crate::args::{parse_lines, value_after};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopCommand {
    List,
    UpsertItem {
        id: String,
        title_key: String,
        price_points: i64,
        metadata_json: Option<String>,
    },
}

pub fn parse(values: &[String]) -> Result<ShopCommand, CliError> {
    match values {
        [sub] if sub == "list" => Ok(ShopCommand::List),
        [sub, action, id, rest @ ..] if sub == "item" && action == "upsert" => upsert(id, rest),
        _ => Err(CliError::message(usage())),
    }
}

fn upsert(id: &str, values: &[String]) -> Result<ShopCommand, CliError> {
    if values.len() != 4 && values.len() != 6 {
        return Err(CliError::message(usage()));
    }
    let mut title_key = String::new();
    let mut price_points = None;
    let mut metadata_json = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--title-key" => title_key = value_after(values, index, "--title-key")?,
            "--price" => price_points = Some(parse_lines(&value_after(values, index, "--price")?)?),
            "--metadata-json" => {
                metadata_json = Some(value_after(values, index, "--metadata-json")?)
            }
            _ => return Err(CliError::message(usage())),
        }
        index += 2;
    }
    Ok(ShopCommand::UpsertItem {
        id: id.to_string(),
        title_key,
        price_points: price_points.ok_or_else(|| CliError::message(usage()))?,
        metadata_json,
    })
}

fn usage() -> &'static str {
    "usage: lkjmc shop list | shop item upsert ITEM --title-key KEY --price POINTS [--metadata-json JSON]"
}
