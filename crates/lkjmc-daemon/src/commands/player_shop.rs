mod player_shop_adventure;

use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::player_shop_delivery::{
    delivery_executor, is_adventure_delivery, preflight_public_adventure_purchase,
};
use crate::dispatch as api;
use crate::support::instance_helpers::{body_string, store, with_connection};

pub(crate) use crate::commands::player_shop_delivery::supported_delivery;

type Response = lkjmc_core::command::CommandResponse;

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let items = store(lkjmc_store::shop::list_items(client))?
            .into_iter()
            .map(|item| {
                let delivery = item.metadata.get("delivery").cloned().unwrap_or(Value::Null);
                let available = supported_delivery(&item.metadata);
                json!({"id": item.id, "titleKey": item.title_key,
                    "category": item.metadata.get("category").and_then(Value::as_str).unwrap_or("misc"),
                    "pricePoints": item.price_points, "deliveryAvailable": available,
                    "deliveryKind": delivery_executor(&item.metadata).unwrap_or(""),
                    "disabledReason": if available { "" } else { "menu.disabled.shop-delivery" },
                    "delivery": delivery})
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"items": items})))
    })
}

pub fn purchase(state: &AppState, request: CommandEnvelope) -> Response {
    if let Err(response) = preflight_public_adventure_purchase(&request) {
        return response;
    }
    with_connection(state, request, |state, request, client| {
        let player = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let item_id = body_string(&request.body, "itemId")?;
        let correlation = parse_uuid(&request, "correlationId")?;
        store(lkjmc_store::player::insert_identity(client, player, &name))?;
        if let Some(body) =
            crate::commands::adventure_api::replay_purchase(client, player, correlation)?
        {
            return Ok(player_shop_adventure::replay(request, body, correlation));
        }
        if let Some(replay) = store(lkjmc_store::shop::replay(client, player, correlation))? {
            return Ok(purchase_response(request, replay));
        }
        let Some(item) = store(lkjmc_store::shop::get_item(client, &item_id))? else {
            return Ok(error(request, "shop.item_not_found", "shop item not found"));
        };
        if is_adventure_delivery(&item.metadata) {
            return player_shop_adventure::purchase(
                state,
                request,
                client,
                player,
                &name,
                &item,
                correlation,
            );
        }
        if delivery_executor(&item.metadata) == Some("minecraft-item")
            && !lkjmc_store::shop::valid_minecraft_item(&item.metadata)
        {
            return Ok(error(
                request,
                "shop.invalid_material",
                "invalid minecraft item delivery",
            ));
        }
        if !supported_delivery(&item.metadata) {
            return Ok(error(
                request,
                "shop.unsupported_delivery",
                "unsupported shop delivery",
            ));
        }
        let purchase = match lkjmc_store::shop::purchase(client, player, &item, correlation) {
            Ok(purchase) => purchase,
            Err(lkjmc_store::error::StoreError::InvalidState(message))
                if message == "insufficient points" =>
            {
                return Ok(error(
                    request,
                    "shop.insufficient_points",
                    "not enough points",
                ));
            }
            Err(error) => return Err(error.to_string()),
        };
        if !purchase.duplicate {
            record_purchase_achievement(client, player, &name)?;
        }
        Ok(purchase_response(request, purchase))
    })
}

pub fn refund(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let refunded = store(lkjmc_store::shop::refund_purchase(
            client,
            parse_uuid(&request, "playerUuid")?,
            parse_uuid(&request, "correlationId")?,
            &body_string(&request.body, "reason")?,
        ))?;
        Ok(api::ok(request, json!({"refunded": refunded})))
    })
}

pub fn upsert_item(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let price = request
            .body
            .get("pricePoints")
            .and_then(Value::as_i64)
            .ok_or_else(|| "missing number field: pricePoints".to_string())?;
        let item_id = body_string(&request.body, "itemId")?;
        store(lkjmc_store::shop::upsert_item_with_metadata(
            client,
            &item_id,
            &body_string(&request.body, "titleKey")?,
            price,
            request
                .body
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| json!({})),
        ))?;
        Ok(api::ok(request, json!({"itemId": item_id})))
    })
}

pub(super) fn record_purchase_achievement(
    client: &mut postgres::Client,
    player: Uuid,
    name: &str,
) -> Result<(), String> {
    store(lkjmc_store::achievement::apply_event_for_player(
        client,
        player,
        Some(name),
        "shop-purchase",
        1,
        None,
    ))?;
    Ok(())
}

pub(crate) fn purchase_response(
    request: CommandEnvelope,
    purchase: lkjmc_store::shop::Purchase,
) -> Response {
    let correlation = request
        .body
        .get("correlationId")
        .cloned()
        .unwrap_or(Value::Null);
    let delivery = if purchase.duplicate {
        Value::Null
    } else {
        purchase
            .item
            .metadata
            .get("delivery")
            .cloned()
            .unwrap_or(Value::Null)
    };
    api::ok(
        request,
        json!({"itemId": purchase.item.id,
        "pricePoints": purchase.item.price_points, "correlationId": correlation,
        "duplicate": purchase.duplicate, "refundable": purchase.refundable,
        "delivery": delivery,
        "deliveryStatus": if purchase.duplicate { "settled-replay" } else { "pending-delivery" }}),
    )
}

fn error(request: CommandEnvelope, code: &str, message: &str) -> Response {
    api::error(request, code, message, false)
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
