use serde_json::Value;

const SENSITIVE_KEYS: &[&str] = &[
    "authorization",
    "bearer",
    "token",
    "cookie",
    "csrf",
    "forwarding",
    "rcon",
    "session",
    "profile",
    "password",
    "databaseurl",
    "url",
    "secret",
];
const URL_PREFIXES: &[&str] = &["http://", "https://", "postgres://", "postgresql://"];

pub(crate) fn json_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut redacted = value.clone();
    redact_value(None, &mut redacted);
    serde_json::to_vec_pretty(&redacted).map_err(|error| error.to_string())
}

pub(crate) fn text_bytes(value: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(value);
    text.lines()
        .map(redact_line)
        .collect::<Vec<_>>()
        .join("\n")
        .into_bytes()
}

pub(crate) fn contains_sensitive_canary(value: &[u8]) -> bool {
    let lower = value.iter().map(u8::to_ascii_lowercase).collect::<Vec<_>>();
    if lower.windows(7).any(|window| window == b"-canary") {
        return true;
    }
    URL_PREFIXES.iter().any(|prefix| {
        lower
            .windows(prefix.len())
            .position(|window| window == prefix.as_bytes())
            .is_some_and(|start| {
                lower[start + prefix.len()..]
                    .split(|byte| byte.is_ascii_whitespace())
                    .next()
                    .is_some_and(|url| url.contains(&b'@') && url.contains(&b':'))
            })
    })
}

fn redact_value(key: Option<&str>, value: &mut Value) {
    if key.is_some_and(sensitive_key) {
        *value = Value::String("[REDACTED]".into());
        return;
    }
    match value {
        Value::Object(values) => {
            for (key, value) in values {
                redact_value(Some(key), value);
            }
        }
        Value::Array(values) => {
            for value in values {
                redact_value(None, value);
            }
        }
        Value::String(text) if is_url(text) => *text = "[REDACTED_URL]".into(),
        _ => {}
    }
}

fn redact_line(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if SENSITIVE_KEYS.iter().any(|key| lower.contains(key)) {
        return "[REDACTED]".into();
    }
    line.split_whitespace()
        .map(|part| if is_url(part) { "[REDACTED_URL]" } else { part })
        .collect::<Vec<_>>()
        .join(" ")
}

fn sensitive_key(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|value| value.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    SENSITIVE_KEYS
        .iter()
        .any(|value| normalized.contains(value))
}

fn is_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    URL_PREFIXES.iter().any(|prefix| lower.starts_with(prefix)) || lower.contains("://")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn every_secret_class_is_removed() {
        assert!(contains_sensitive_canary(b"BEARER OBS-TOKEN-CANARY"));
        let value = json!({
            "authorization":"Bearer obs-token-canary", "cookie":"obs-cookie-canary",
            "csrfToken":"obs-csrf-canary", "forwardingSecret":"obs-forwarding-canary",
            "rconPassword":"obs-rcon-canary", "sessionId":"obs-session-canary",
            "profilePayload":"obs-profile-canary", "databaseUrl":"postgresql://obs:password@localhost/obs"
        });
        let output = json_bytes(&value).unwrap_or_default();
        assert!(!contains_sensitive_canary(&output));
        assert_eq!(
            String::from_utf8_lossy(&output)
                .matches("[REDACTED]")
                .count(),
            8
        );
    }
}
