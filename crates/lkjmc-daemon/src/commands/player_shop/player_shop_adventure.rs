use lkjmc_core::command::CommandEnvelope;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
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
    let body = nested_body(&request.body, name, item, correlation);
    let mut nested = request.clone();
    nested.command = "adventure.purchase".to_string();
    nested.body = body;
    let response = crate::commands::adventure_api::handle(state, nested);
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
    Ok(api::ok(request, body))
}

fn nested_body(
    request: &Value,
    name: &str,
    item: &lkjmc_store::shop::ShopItem,
    correlation: Uuid,
) -> Value {
    let mut body = request.clone();
    body["playerName"] = Value::String(name.to_string());
    body["cost"] = Value::Number(item.price_points.into());
    body["acceptMinecraftEula"] = Value::Bool(true);
    body["correlationId"] = Value::String(correlation.to_string());
    body["adventureId"] = Value::String(
        item.metadata
            .pointer("/delivery/adventureId")
            .and_then(Value::as_str)
            .unwrap_or("end-expedition")
            .to_string(),
    );
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_adventure_keeps_the_outer_correlation() {
        let correlation = Uuid::nil();
        let item = lkjmc_store::shop::ShopItem {
            id: "adventure-end".to_string(),
            title_key: "shop.adventure-end".to_string(),
            price_points: 20,
            metadata: json!({"delivery": {"adventureId": "end-expedition"}}),
        };
        let body = nested_body(&json!({"playerUuid": "buyer"}), "Buyer", &item, correlation);
        assert_eq!(body["correlationId"], correlation.to_string());
        assert_eq!(body["adventureId"], "end-expedition");
        assert_eq!(body["cost"], 20);
    }
}
