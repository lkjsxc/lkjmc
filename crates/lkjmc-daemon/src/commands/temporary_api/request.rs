use lkjmc_core::temporary::CleanupPolicy;
use serde_json::Value;

pub fn string(body: &Value, field: &'static str) -> Result<String, String> {
    body.get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("missing string field: {field}"))
}

pub fn optional_string(body: &Value, field: &'static str, fallback: String) -> String {
    body.get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or(fallback)
}

pub fn u32_field(body: &Value, field: &'static str, fallback: u32) -> Result<u32, String> {
    let Some(value) = body.get(field) else {
        return Ok(fallback);
    };
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("{field} must be a positive integer"))
}

pub fn cleanup_policy(body: &Value) -> Result<CleanupPolicy, String> {
    match body
        .get("cleanupPolicy")
        .and_then(Value::as_str)
        .unwrap_or("delete")
    {
        "delete" => Ok(CleanupPolicy::Delete),
        "archive" => Ok(CleanupPolicy::Archive),
        other => Err(format!("unsupported cleanup policy: {other}")),
    }
}

pub fn require_eula(body: &Value) -> Result<(), String> {
    if body
        .get("acceptMinecraftEula")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err("temporary Folia instances require acceptMinecraftEula=true".to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn cleanup_policy_rejects_unknown_values() {
        assert_eq!(cleanup_policy(&json!({})), Ok(CleanupPolicy::Delete));
        assert!(cleanup_policy(&json!({"cleanupPolicy":"copy"})).is_err());
    }
}
