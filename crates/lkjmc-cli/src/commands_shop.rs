use serde_json::{json, Value};

use crate::args_shop::ShopCommand;
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: ShopCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        ShopCommand::List => daemon_command(
            socket,
            "player.shop.list",
            json!({}),
            json_output,
            "ok shop list",
        ),
        ShopCommand::UpsertItem {
            id,
            title_key,
            price_points,
            metadata_json,
        } => daemon_command(
            socket,
            "shop.item.upsert",
            upsert_body(id, title_key, price_points, metadata_json)?,
            json_output,
            "ok shop item upsert",
        ),
    }
}

fn upsert_body(
    id: String,
    title_key: String,
    price_points: i64,
    metadata_json: Option<String>,
) -> Result<Value, CliError> {
    let metadata = match metadata_json {
        Some(value) => serde_json::from_str::<Value>(&value)?,
        None => json!({}),
    };
    Ok(
        json!({"itemId": id, "titleKey": title_key, "pricePoints": price_points, "metadata": metadata}),
    )
}
