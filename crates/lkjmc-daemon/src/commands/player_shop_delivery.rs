use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::Value;

use crate::commands::adventure_confirmation;

pub(crate) fn supported_delivery(item_id: &str, metadata: &Value) -> bool {
    lkjmc_store::shop::is_canonical_adventure_delivery(item_id, metadata)
        || (delivery_executor(metadata) == Some("minecraft-item")
            && lkjmc_store::shop::valid_minecraft_item(metadata))
}

pub(crate) fn preflight_public_adventure_purchase(
    request: &CommandEnvelope,
) -> Result<(), CommandResponse> {
    if request.body.get("itemId").and_then(Value::as_str) == Some("adventure-end-expedition")
        && !adventure_confirmation::accepted(&request.body)
    {
        return Err(adventure_confirmation::required(request.clone()));
    }
    Ok(())
}

pub(crate) fn adventure_request(
    request: &CommandEnvelope,
    name: &str,
    item: &lkjmc_store::shop::ShopItem,
) -> Result<CommandEnvelope, CommandResponse> {
    if !lkjmc_store::shop::is_canonical_adventure_delivery(&item.id, &item.metadata) {
        return Err(unsupported_delivery(request.clone()));
    }
    if !adventure_confirmation::accepted(&request.body) {
        return Err(adventure_confirmation::required(request.clone()));
    }
    let mut nested = request.clone();
    nested.command = "adventure.purchase".to_string();
    nested.body["playerName"] = Value::String(name.to_string());
    nested.body["cost"] = Value::Number(item.price_points.into());
    nested.body["adventureId"] = Value::String("end-expedition".to_string());
    Ok(nested)
}

pub(crate) fn is_adventure_delivery(item: &lkjmc_store::shop::ShopItem) -> bool {
    lkjmc_store::shop::is_canonical_adventure_delivery(&item.id, &item.metadata)
}

pub(crate) fn delivery_executor(metadata: &Value) -> Option<&str> {
    metadata
        .pointer("/delivery/executor")
        .and_then(Value::as_str)
}

fn unsupported_delivery(request: CommandEnvelope) -> CommandResponse {
    crate::dispatch::error(
        request,
        "shop.unsupported_delivery",
        "unsupported shop delivery",
        false,
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::supported_delivery;

    #[test]
    fn only_canonical_item_has_adventure_delivery() {
        let canonical = json!({"delivery":{"executor":"adventure","adventureId":"end-expedition"}});
        assert!(supported_delivery("adventure-end-expedition", &canonical));
        assert!(!supported_delivery("custom", &canonical));
        assert!(!supported_delivery(
            "adventure-end-expedition",
            &json!({"delivery":{"executor":"adventure-end-expedition"}})
        ));
    }
}
