use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_client};

type Response = lkjmc_core::command::CommandResponse;

pub fn list(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let items = store(lkjmc_store::shop::list_items(client))?
            .into_iter()
            .map(|item| {
                json!({
                    "id": item.id,
                    "titleKey": item.title_key,
                    "pricePoints": item.price_points,
                    "deliveryAvailable": supported_delivery(&item.metadata),
                    "delivery": item.metadata.get("delivery").cloned().unwrap_or(Value::Null)
                })
            })
            .collect::<Vec<_>>();
        Ok(api::ok(request, json!({"items": items})))
    })
}

pub fn purchase(state: &AppState, request: CommandEnvelope) -> Response {
    with_client(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let name = body_string(&request.body, "name")?;
        let item_id = body_string(&request.body, "itemId")?;
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &name,
        ))?;
        let item = store(lkjmc_store::shop::get_item(client, &item_id))?
            .ok_or_else(|| format!("shop item not found: {item_id}"))?;
        if !supported_delivery(&item.metadata) {
            return Err("shop item has no supported delivery".to_string());
        }
        let spent = store(lkjmc_store::points::spend(
            client,
            player_uuid,
            item.price_points,
            "shop.purchase",
        ))?;
        if !spent {
            return Err("not enough points".to_string());
        }
        store(lkjmc_store::shop::record_purchase(
            client,
            player_uuid,
            &item,
        ))?;
        store(lkjmc_store::achievement::grant(
            client,
            player_uuid,
            "first-purchase",
            "achievement.first-purchase",
        ))?;
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
    with_client(state, request, |_state, request, client| {
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

fn supported_delivery(metadata: &Value) -> bool {
    metadata
        .get("delivery")
        .and_then(Value::as_object)
        .is_some_and(|delivery| {
            delivery.get("executor").and_then(Value::as_str) == Some("minecraft-item")
                && delivery.get("material").and_then(Value::as_str).is_some()
        })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
