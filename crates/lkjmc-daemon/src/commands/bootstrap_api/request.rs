use serde_json::Value;

use crate::app::AppState;

pub(super) fn validate(state: &AppState, body: &Value) -> Result<(), String> {
    let profile = body
        .get("profile")
        .and_then(Value::as_str)
        .unwrap_or("playable");
    if profile != "playable" {
        return Err(format!("unsupported bootstrap profile: {profile}"));
    }
    state.runtime_config()?;
    if let Some(value) = body.get("javaBindHost") {
        nonempty(value, "javaBindHost")?;
    }
    if let Some(value) = body.get("javaPublicHost") {
        nonempty(value, "javaPublicHost")?;
    }
    for field in ["javaPort", "bedrockPort"] {
        if let Some(value) = body.get(field) {
            port(value, field)?;
        }
    }
    if let Some(mode) = body.get("bedrock").and_then(Value::as_str) {
        if !matches!(mode, "auto" | "enabled" | "disabled") {
            return Err(format!("unsupported bedrock mode: {mode}"));
        }
    }
    Ok(())
}

fn nonempty(value: &Value, field: &str) -> Result<(), String> {
    if value.as_str().is_some_and(|value| !value.is_empty()) {
        Ok(())
    } else {
        Err(format!("{field} must not be empty"))
    }
}

fn port(value: &Value, field: &str) -> Result<(), String> {
    match value.as_u64() {
        Some(1..=65535) => Ok(()),
        Some(_) => Err(format!("{field} must be between 1 and 65535")),
        None => Err(format!("{field} must be an integer port")),
    }
}
