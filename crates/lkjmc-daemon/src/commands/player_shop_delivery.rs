use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::Value;

use crate::commands::adventure_confirmation;

pub(crate) fn supported_delivery(metadata: &Value) -> bool {
    match delivery_executor(metadata) {
        Some("minecraft-item") => lkjmc_store::shop::valid_minecraft_item(metadata),
        Some("adventure") => matches!(
            metadata
                .pointer("/delivery/adventureId")
                .and_then(Value::as_str),
            Some("end-expedition")
        ),
        Some("adventure-end-expedition") => true,
        _ => false,
    }
}

pub(crate) fn adventure_request(
    request: &CommandEnvelope,
    name: &str,
    item: &lkjmc_store::shop::ShopItem,
) -> Result<CommandEnvelope, CommandResponse> {
    if !adventure_confirmation::accepted(&request.body) {
        return Err(adventure_confirmation::required(request.clone()));
    }
    let adventure_id = item
        .metadata
        .pointer("/delivery/adventureId")
        .and_then(Value::as_str)
        .unwrap_or("end-expedition");
    let mut nested = request.clone();
    nested.command = "adventure.purchase".to_string();
    nested.body["playerName"] = Value::String(name.to_string());
    nested.body["cost"] = Value::Number(item.price_points.into());
    nested.body["adventureId"] = Value::String(adventure_id.to_string());
    Ok(nested)
}

pub(crate) fn is_adventure_delivery(metadata: &Value) -> bool {
    supported_delivery(metadata)
        && matches!(
            delivery_executor(metadata),
            Some("adventure") | Some("adventure-end-expedition")
        )
}

pub(crate) fn delivery_executor(metadata: &Value) -> Option<&str> {
    metadata
        .pointer("/delivery/executor")
        .and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::supported_delivery;

    #[test]
    fn invalid_minecraft_item_metadata_is_disabled_before_settlement() {
        assert!(!supported_delivery(&json!({"delivery": {
            "executor": "minecraft-item", "material": "NOT_A_MATERIAL", "amount": 1
        }})));
        assert!(!supported_delivery(&json!({"delivery": {
            "executor": "minecraft-item", "material": "STONE", "amount": 65
        }})));
    }
}
