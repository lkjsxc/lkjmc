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
    "redacted",
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

pub(super) fn sanitize_attributes(
    values: BTreeMap<String, Value>,
) -> (BTreeMap<String, Value>, bool) {
    let mut output = BTreeMap::new();
    let mut redacted = false;
    for (key, value) in values {
        let valid = ATTRIBUTE_KEYS.contains(&key.as_str())
            && match &value {
                Value::Bool(_) | Value::Number(_) | Value::Null => true,
                Value::String(text) => text.len() <= 128 && safe(text),
                _ => false,
            };
        if valid && key != "redacted" && output.len() < 11 {
            output.insert(key, value);
        } else {
            redacted = true;
        }
    }
    if redacted {
        output.insert("redacted".into(), Value::Bool(true));
    }
    (output, redacted)
}

pub(super) fn required(value: String, maximum: usize) -> (String, bool) {
    if !value.is_empty() && value.len() <= maximum && safe(&value) {
        (value, false)
    } else {
        ("[redacted]".into(), true)
    }
}

pub(super) fn optional(value: Option<String>, maximum: usize) -> (Option<String>, bool) {
    match value {
        Some(item) if !item.is_empty() && item.len() <= maximum && safe(&item) => {
            (Some(item), false)
        }
        Some(_) => (None, true),
        None => (None, false),
    }
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
            assert!(required(value.into(), 128).1);
        }
    }
}
