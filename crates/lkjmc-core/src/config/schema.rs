use serde_json::{json, Value};

pub const REQUIRED_JAVA_FIELDS: &[&str] = &[
    "LKJMC_DAEMON_HTTP_URL",
    "LKJMC_DAEMON_HTTP_TOKEN",
    "LKJMC_DAEMON_HTTP_TOKEN_FILE",
    "LKJMC_INSTANCE_ID",
    "LKJMC_PLATFORM_ROLE",
    "LKJMC_DEFAULT_LOCALE",
];

pub fn java_contract() -> Value {
    json!({
        "owner": "lkjmc-core",
        "fields": REQUIRED_JAVA_FIELDS,
        "diagnostics": [
            "daemon.not_configured",
            "daemon.token_missing",
            "daemon.token_unreadable",
            "schema.invalid_url",
            "schema.invalid_instance_id",
            "schema.invalid_locale"
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_contract_names_required_fields() {
        let encoded = java_contract().to_string();
        for field in REQUIRED_JAVA_FIELDS {
            assert!(encoded.contains(field), "{field}");
        }
    }
}
