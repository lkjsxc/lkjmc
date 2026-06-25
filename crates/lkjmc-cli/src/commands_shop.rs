use serde_json::json;

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
        } => daemon_command(
            socket,
            "shop.item.upsert",
            json!({"itemId": id, "titleKey": title_key, "pricePoints": price_points}),
            json_output,
            "ok shop item upsert",
        ),
    }
}
