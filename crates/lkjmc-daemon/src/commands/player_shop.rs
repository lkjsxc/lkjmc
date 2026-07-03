use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let items = store(lkjmc_store::shop::list_items(client))?
            .into_iter()
            .map(|item| {
                let delivery = item.metadata.get("delivery").cloned().unwrap_or(Value::Null);
                let delivery_available = supported_delivery(&item.metadata);
                json!({
                    "id": item.id,
                    "titleKey": item.title_key,
                    "category": item.metadata.get("category").and_then(Value::as_str).unwrap_or("misc"),
                    "pricePoints": item.price_points,
                    "deliveryAvailable": delivery_available,
                    "deliveryKind": delivery_executor(&item.metadata).unwrap_or(""),
                    "disabledReason": if delivery_available {""} else {"menu.disabled.shop-delivery"},
                    "delivery": delivery
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"items": items})))
    })
}

pub fn purchase(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let item_id = body_string(&request.body, "itemId")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        let Some(item) = store(lkjmc_store::shop::get_item(client, &item_id))? else {
            return Ok(api::error(
                request,
                "shop.item_not_found",
                format!("shop item not found: {item_id}"),
                false,
            ));
        };
        if is_adventure_delivery(&item.metadata) {
            return adventure_purchase(state, request, client, player_uuid, &name, &item);
        }
        if !supported_delivery(&item.metadata) {
            return Ok(api::error(
                request,
                "shop.unsupported_delivery",
                "shop item has no supported delivery",
                false,
            ));
        }
        let spent = store(lkjmc_store::points::spend(
            client,
            player_uuid,
            item.price_points,
            "shop.purchase",
        ))?;
        if !spent {
            return Ok(api::error(
                request,
                "shop.insufficient_points",
                "not enough points",
                false,
            ));
        }
        record_success(client, player_uuid, Some(&name), &item)?;
        Ok(api::ok(
            request,
            json!({
                "itemId": item.id,
                "pricePoints": item.price_points,
                "delivery": item.metadata.get("delivery").cloned().unwrap_or(Value::Null)
            }),
        ))
    })
}

pub fn upsert_item(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let item_id = body_string(&request.body, "itemId")?;
        let title_key = body_string(&request.body, "titleKey")?;
        let price = request
            .body
            .get("pricePoints")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| "missing number field: pricePoints".to_string())?;
        let metadata = request
            .body
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default()));
        store(lkjmc_store::shop::upsert_item_with_metadata(
            client, &item_id, &title_key, price, metadata,
        ))?;
        Ok(api::ok(request, json!({"itemId": item_id})))
    })
}

fn adventure_purchase(
    state: &AppState,
    request: CommandEnvelope,
    client: &mut postgres::Client,
    player_uuid: Uuid,
    name: &str,
    item: &lkjmc_store::shop::ShopItem,
) -> Result<Response, String> {
    let mut body = request.body.clone();
    body["playerName"] = Value::String(name.to_string());
    body["cost"] = Value::Number(item.price_points.into());
    body["acceptMinecraftEula"] = Value::Bool(true);
    let adventure_id = item
        .metadata
        .pointer("/delivery/adventureId")
        .and_then(Value::as_str)
        .unwrap_or("end-expedition");
    body["adventureId"] = Value::String(adventure_id.to_string());
    let mut nested = request.clone();
    nested.command = "adventure.purchase".to_string();
    nested.body = body;
    let response = crate::commands::adventure_api::handle(state, nested);
    if !response.ok {
        return Ok(response);
    }
    record_success(client, player_uuid, Some(name), item)?;
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

fn record_success(
    client: &mut postgres::Client,
    player_uuid: Uuid,
    player_name: Option<&str>,
    item: &lkjmc_store::shop::ShopItem,
) -> Result<(), String> {
    store(lkjmc_store::shop::record_purchase(
        client,
        player_uuid,
        item,
    ))?;
    store(lkjmc_store::achievement::apply_event_for_player(
        client,
        player_uuid,
        player_name,
        "shop-purchase",
        1,
        None,
    ))?;
    Ok(())
}

pub(crate) fn supported_delivery(metadata: &Value) -> bool {
    match delivery_executor(metadata) {
        Some("minecraft-item") => metadata
            .pointer("/delivery/material")
            .and_then(Value::as_str)
            .is_some(),
        Some("adventure") => metadata
            .pointer("/delivery/adventureId")
            .and_then(Value::as_str)
            .is_some(),
        Some("adventure-end-expedition") => true,
        _ => false,
    }
}

fn is_adventure_delivery(metadata: &Value) -> bool {
    matches!(
        delivery_executor(metadata),
        Some("adventure") | Some("adventure-end-expedition")
    )
}

fn delivery_executor(metadata: &Value) -> Option<&str> {
    metadata
        .pointer("/delivery/executor")
        .and_then(Value::as_str)
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
