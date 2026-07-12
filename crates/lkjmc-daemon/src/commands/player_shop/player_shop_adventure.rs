use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::player_shop_delivery::adventure_request;
use crate::dispatch as api;

use super::record_purchase_achievement;

type Response = lkjmc_core::command::CommandResponse;

pub(super) fn purchase(
    state: &AppState,
    request: CommandEnvelope,
    client: &mut postgres::Client,
    player: Uuid,
    name: &str,
    item: &lkjmc_store::shop::ShopItem,
    correlation: Uuid,
) -> Result<Response, String> {
    let mut nested = match adventure_request(&request, name, item) {
        Ok(nested) => nested,
        Err(response) => return Ok(response),
    };
    nested.body["correlationId"] = Value::String(correlation.to_string());
    let response = crate::dispatch::dispatch_internal(state, nested);
    if !response.ok {
        return Ok(response);
    }
    let mut body = response.body.unwrap_or_else(|| json!({}));
    let duplicate = body
        .get("duplicate")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !duplicate {
        record_purchase_achievement(client, player, name)?;
    }
    body["itemId"] = Value::String(item.id.clone());
    body["pricePoints"] = Value::Number(item.price_points.into());
    body["correlationId"] = Value::String(correlation.to_string());
    body["duplicate"] = Value::Bool(duplicate);
    body["refundable"] = Value::Bool(false);
    body["delivery"] = if duplicate {
        Value::Null
    } else {
        item.metadata
            .get("delivery")
            .cloned()
            .unwrap_or(Value::Null)
    };
    body["deliveryStatus"] = Value::String(if duplicate {
        "settled-replay".to_string()
    } else {
        "pending-delivery".to_string()
    });
    Ok(api::ok(request, body))
}

pub(super) fn replay(request: CommandEnvelope, mut body: Value, correlation: Uuid) -> Response {
    body["correlationId"] = Value::String(correlation.to_string());
    body["duplicate"] = Value::Bool(true);
    body["refundable"] = Value::Bool(false);
    body["delivery"] = Value::Null;
    body["deliveryStatus"] = Value::String("settled-replay".to_string());
    api::ok(request, body)
}
