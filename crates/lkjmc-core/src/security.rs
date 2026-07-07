use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRotationPlan {
    pub token_file: Option<String>,
    pub daemon_hot_swap: bool,
    pub consumer_action: String,
    pub verification: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRotationStatus {
    pub configured: bool,
    pub token_file: Option<String>,
    pub fingerprint: Option<String>,
    pub scoped_token_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopedTokenCreateResult {
    pub credential_id: String,
    pub surface: String,
    pub scopes: Vec<String>,
    pub token: String,
    pub fingerprint: String,
}

pub fn rotation_plan(token_file: Option<String>) -> TokenRotationPlan {
    TokenRotationPlan {
        token_file,
        daemon_hot_swap: true,
        consumer_action: "restart-managed-jvm-consumers".to_string(),
        verification: vec![
            "new-token-accepted".to_string(),
            "old-token-rejected".to_string(),
            "audit-written-with-fingerprint".to_string(),
        ],
    }
}

pub fn token_hash(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

pub fn token_fingerprint(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!(
        "sha256:{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3]
    )
}

pub fn redacted_fingerprint(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let mut value = 0_u64;
    for byte in bytes {
        value = value.rotate_left(5) ^ u64::from(*byte);
    }
    Some(format!("fp:{value:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_does_not_expose_token() {
        let token = b"secret-token";
        let fp = redacted_fingerprint(token).unwrap_or_default();
        assert!(fp.starts_with("fp:"));
        assert!(!fp.contains("secret"));
    }

    #[test]
    fn plan_has_no_secret_value() {
        let plan = rotation_plan(Some("/etc/lkjmc/token".to_string()));
        assert_eq!(plan.token_file.as_deref(), Some("/etc/lkjmc/token"));
        assert!(plan.daemon_hot_swap);
    }
}
