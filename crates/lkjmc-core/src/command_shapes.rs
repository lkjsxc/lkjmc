use serde_json::{json, Map, Value};

use crate::command_registry::ValueType;

pub(crate) fn matches_type(value: &Value, value_type: &ValueType) -> bool {
    match value_type {
        ValueType::Array => value.is_array(),
        ValueType::Boolean => value.is_boolean(),
        ValueType::EmptyObject => empty_object(value),
        ValueType::Integer => value.as_i64().is_some(),
        ValueType::Number => value.is_number(),
        ValueType::RconConfig => rcon_config(value),
        ValueType::ShopMetadata => shop_metadata(value),
        ValueType::String => value.is_string(),
        ValueType::WorldLocation => world_location(value),
    }
}

fn empty_object(value: &Value) -> bool {
    value.as_object().is_some_and(Map::is_empty)
}

fn rcon_config(value: &Value) -> bool {
    let Some(fields) = value.as_object() else {
        return false;
    };
    keys_are(fields, &["host", "password", "port"])
        && string(fields, "password")
        && integer(fields, "port")
        && optional_string(fields, "host")
}

fn world_location(value: &Value) -> bool {
    let Some(fields) = value.as_object() else {
        return false;
    };
    keys_are(fields, &["world", "x", "y", "z"])
        && string(fields, "world")
        && number(fields, "x")
        && number(fields, "y")
        && number(fields, "z")
}

fn shop_metadata(value: &Value) -> bool {
    let Some(fields) = value.as_object() else {
        return false;
    };
    if !keys_are(fields, &["category", "delivery"]) || !optional_string(fields, "category") {
        return false;
    }
    let Some(delivery) = fields.get("delivery") else {
        return true;
    };
    let Some(delivery) = delivery.as_object() else {
        return false;
    };
    match delivery.get("executor").and_then(Value::as_str) {
        Some("minecraft-item") => {
            keys_are(delivery, &["executor", "material", "amount"])
                && string(delivery, "material")
                && integer(delivery, "amount")
        }
        Some("adventure") => {
            value == &json!({"delivery":{"executor":"adventure","adventureId":"end-expedition"}})
        }
        _ => false,
    }
}

fn keys_are(fields: &Map<String, Value>, allowed: &[&str]) -> bool {
    fields.keys().all(|key| allowed.contains(&key.as_str()))
}

fn string(fields: &Map<String, Value>, key: &str) -> bool {
    fields.get(key).and_then(Value::as_str).is_some()
}

fn optional_string(fields: &Map<String, Value>, key: &str) -> bool {
    fields.get(key).is_none_or(Value::is_string)
}

fn integer(fields: &Map<String, Value>, key: &str) -> bool {
    fields.get(key).and_then(Value::as_i64).is_some()
}

fn number(fields: &Map<String, Value>, key: &str) -> bool {
    fields.get(key).is_some_and(Value::is_number)
}
