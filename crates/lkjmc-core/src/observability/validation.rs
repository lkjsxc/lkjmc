use std::collections::BTreeMap;

use serde_json::Value;

const ATTRIBUTE_KEYS: &[&str] = &[
    "command",
    "serverId",
    "route",
    "runtime",
    "fault",
    "queue",
    "reason",
    "migration",
    "retention",
    "bundle",
    "transport",
    "source",
];
const SENSITIVE_MARKERS: &[&str] = &[
    "bearer ",
    "authorization:",
    "password=",
    "secret=",
    "token=",
    "cookie=",
    "csrf=",
    "obs-token-canary",
    "obs-cookie-canary",
    "obs-csrf-canary",
    "obs-forwarding-canary",
    "obs-rcon-canary",
    "obs-session-canary",
    "obs-profile-canary",
];

pub(super) fn attributes(values: &BTreeMap<String, Value>) -> Result<(), String> {
    if values.len() > 12 {
        return Err("too many event attributes".into());
    }
    for (key, value) in values {
        if !ATTRIBUTE_KEYS.contains(&key.as_str()) {
            return Err(format!("event attribute not allowed: {key}"));
        }
        match value {
            Value::Bool(_) | Value::Number(_) | Value::Null => {}
            Value::String(text) if text.len() <= 128 && safe(text) => {}
            _ => return Err(format!("event attribute is unbounded or sensitive: {key}")),
        }
    }
    Ok(())
}

pub(super) fn bounded(name: &str, value: String, maximum: usize) -> Result<String, String> {
    if value.is_empty() || value.len() > maximum || !safe(&value) {
        Err(format!("{name} is not bounded or is sensitive"))
    } else {
        Ok(value)
    }
}

pub(super) fn optional_bounded(
    name: &str,
    value: Option<String>,
    maximum: usize,
) -> Result<Option<String>, String> {
    value.map(|item| bounded(name, item, maximum)).transpose()
}

fn safe(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    !lower.contains("://")
        && !SENSITIVE_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_urls_and_secret_canaries() {
        for value in ["https://user:pass@example.test", "Bearer obs-token-canary"] {
            assert!(bounded("field", value.into(), 128).is_err());
        }
    }
}
