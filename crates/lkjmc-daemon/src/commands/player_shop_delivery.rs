use serde_json::Value;

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
