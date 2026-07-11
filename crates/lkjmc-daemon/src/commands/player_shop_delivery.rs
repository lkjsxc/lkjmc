use serde_json::Value;

pub(crate) fn supported_delivery(metadata: &Value) -> bool {
    match delivery_executor(metadata) {
        Some("minecraft-item") => lkjmc_store::shop::valid_minecraft_item(metadata),
        Some("adventure") => metadata
            .pointer("/delivery/adventureId")
            .and_then(Value::as_str)
            .is_some(),
        Some("adventure-end-expedition") => true,
        _ => false,
    }
}

pub(crate) fn is_adventure_delivery(metadata: &Value) -> bool {
    matches!(
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
