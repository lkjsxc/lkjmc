use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

pub(crate) fn supported_delivery(item_id: &str, metadata: &Value) -> bool {
    lkjmc_store::shop::is_canonical_adventure_delivery(item_id, metadata)
        || (delivery_executor(metadata) == Some("minecraft-item")
            && lkjmc_store::shop::valid_minecraft_item(metadata))
}

pub(crate) fn adventure_request(
    request: &CommandEnvelope,
    name: &str,
    item: &lkjmc_store::shop::ShopItem,
) -> Result<CommandEnvelope, CommandResponse> {
    if !lkjmc_store::shop::is_canonical_adventure_delivery(&item.id, &item.metadata) {
        return Err(unsupported_delivery(request.clone()));
    }
    let body = json!({
        "playerUuid": request.body.get("playerUuid").cloned().unwrap_or(Value::Null),
        "playerName": name,
        "cost": item.price_points,
        "adventureId": "end-expedition"
    });
    Ok(CommandEnvelope {
        request_id: request.request_id.clone(),
        actor: request.actor.clone(),
        command: "adventure.purchase".to_string(),
        body,
    })
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
