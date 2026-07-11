use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::player_shop_delivery::{delivery_executor, is_adventure_delivery};
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

pub(crate) use crate::commands::player_shop_delivery::supported_delivery;
type Response = lkjmc_core::command::CommandResponse;

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let items = store(lkjmc_store::shop::list_items(client))?.into_iter().map(|item| {
            let available = supported_delivery(&item.metadata);
            json!({"id":item.id,"titleKey":item.title_key,
                "category":item.metadata.get("category").and_then(Value::as_str).unwrap_or("misc"),
                "pricePoints":item.price_points,"deliveryAvailable":available,
                "deliveryKind":delivery_executor(&item.metadata).unwrap_or(""),
                "disabledReason":if available {""} else {"menu.disabled.shop-delivery"},
                "delivery":item.metadata.get("delivery").cloned().unwrap_or(Value::Null)})
        }).collect::<Vec<_>>();
        Ok(api::ok(request, json!({"items":items})))
    })
}

pub fn purchase(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let item_id = body_string(&request.body, "itemId")?;
        let correlation_id = parse_uuid(&request, "correlationId")?;
        store(lkjmc_store::player::insert_identity(client, player_uuid, &name))?;
        let Some(item) = store(lkjmc_store::shop::get_item(client, &item_id))? else {
            return Ok(error(request, "shop.item_not_found", "shop item not found"));
        };
        if is_adventure_delivery(&item.metadata) {
            return adventure_purchase(state, request, client, player_uuid, &name, &item);
        }
        if !supported_delivery(&item.metadata) {
            return Ok(error(request, "shop.unsupported_delivery", "unsupported shop delivery"));
        }
        let purchase = match lkjmc_store::shop::purchase(client, player_uuid, &item_id, correlation_id) {
            Ok(Some(purchase)) => purchase,
            Ok(None) => return Ok(error(request, "shop.item_not_found", "shop item not found")),
            Err(lkjmc_store::error::StoreError::InvalidState(message)) if message == "insufficient points" => {
                return Ok(error(request, "shop.insufficient_points", "not enough points"));
            }
            Err(error) => return Err(error.to_string()),
        };
        if !purchase.duplicate { record_success(client, player_uuid, Some(&name))?; }
        Ok(purchase_response(request, &purchase.item, correlation_id, purchase.duplicate, purchase.refunded))
    })
}

pub fn reconcile(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let correlation_id = parse_uuid(&request, "correlationId")?;
        let Some(purchase) = store(lkjmc_store::shop::reconcile_purchase(client, player_uuid, correlation_id))? else {
            return Ok(error(request, "shop.correlation_not_found", "purchase correlation not found"));
        };
        Ok(purchase_response(request, &purchase.item, correlation_id, true, purchase.refunded))
    })
}

pub fn refund(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let refunded = store(lkjmc_store::shop::refund_purchase(client, parse_uuid(&request, "playerUuid")?,
            parse_uuid(&request, "correlationId")?, &body_string(&request.body, "reason")?))?;
        Ok(api::ok(request, json!({"refunded":refunded})))
    })
}

pub fn upsert_item(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let price = request.body.get("pricePoints").and_then(Value::as_i64)
            .ok_or_else(|| "missing number field: pricePoints".to_string())?;
        let item_id = body_string(&request.body, "itemId")?;
        store(lkjmc_store::shop::upsert_item_with_metadata(client, &item_id,
            &body_string(&request.body, "titleKey")?, price, request.body.get("metadata").cloned().unwrap_or_else(|| json!({}))))?;
        Ok(api::ok(request, json!({"itemId":item_id})))
    })
}

fn adventure_purchase(state: &AppState, request: CommandEnvelope, client: &mut postgres::Client, player: Uuid, name: &str, item: &lkjmc_store::shop::ShopItem) -> Result<Response, String> {
    let mut body = request.body.clone();
    body["playerName"] = Value::String(name.to_string()); body["cost"] = Value::Number(item.price_points.into()); body["acceptMinecraftEula"] = Value::Bool(true);
    body["adventureId"] = Value::String(item.metadata.pointer("/delivery/adventureId").and_then(Value::as_str).unwrap_or("end-expedition").to_string());
    let mut nested = request.clone(); nested.command = "adventure.purchase".to_string(); nested.body = body;
    let response = crate::commands::adventure_api::handle(state, nested);
    if !response.ok { return Ok(response); }
    record_success(client, player, Some(name))?;
    let mut body = response.body.unwrap_or_else(|| json!({})); body["itemId"] = Value::String(item.id.clone()); body["pricePoints"] = Value::Number(item.price_points.into()); body["delivery"] = item.metadata.get("delivery").cloned().unwrap_or(Value::Null);
    Ok(api::ok(request, body))
}
fn record_success(client: &mut postgres::Client, player: Uuid, name: Option<&str>) -> Result<(), String> {
    store(lkjmc_store::achievement::apply_event_for_player(client, player, name, "shop-purchase", 1, None))?; Ok(())
}
fn purchase_response(request: CommandEnvelope, item: &lkjmc_store::shop::ShopItem, correlation: Uuid, duplicate: bool, refunded: bool) -> Response {
    api::ok(request, json!({"itemId":item.id,"pricePoints":item.price_points,"correlationId":correlation.to_string(),"duplicate":duplicate,"refunded":refunded,"delivery":item.metadata.get("delivery").cloned().unwrap_or(Value::Null)}))
}
fn error(request: CommandEnvelope, code: &str, message: &str) -> Response { api::error(request, code, message, false) }
fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> { Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string()) }
