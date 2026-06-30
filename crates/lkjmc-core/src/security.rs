use serde::{Deserialize, Serialize};

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
