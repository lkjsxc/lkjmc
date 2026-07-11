use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::store;

type Response = lkjmc_core::command::CommandResponse;

pub(super) fn purchase(
    state: &AppState,
    request: CommandEnvelope,
    client: &mut postgres::Client,
    player: Uuid,
    name: &str,
    item: &lkjmc_store::shop::ShopItem,
) -> Result<Response, String> {
    let mut body = request.body.clone();
    body["playerName"] = Value::String(name.to_string());
    body["cost"] = Value::Number(item.price_points.into());
    body["acceptMinecraftEula"] = Value::Bool(true);
    body["adventureId"] = Value::String(
        item.metadata
            .pointer("/delivery/adventureId")
            .and_then(Value::as_str)
            .unwrap_or("end-expedition")
            .to_string(),
    );
    let mut nested = request.clone();
    nested.command = "adventure.purchase".to_string();
    nested.body = body;
    let response = crate::commands::adventure_api::handle(state, nested);
    if !response.ok {
        return Ok(response);
    }
    record_success(client, player, Some(name))?;
    let mut body = response.body.unwrap_or_else(|| json!({}));
    body["itemId"] = Value::String(item.id.clone());
    body["pricePoints"] = Value::Number(item.price_points.into());
    body["delivery"] = item
        .metadata
        .get("delivery")
        .cloned()
        .unwrap_or(Value::Null);
    Ok(api::ok(request, body))
}

pub(super) fn record_success(
    client: &mut postgres::Client,
    player: Uuid,
    name: Option<&str>,
) -> Result<(), String> {
    store(lkjmc_store::achievement::apply_event_for_player(
        client,
        player,
        name,
        "shop-purchase",
        1,
        None,
    ))?;
    Ok(())
}
