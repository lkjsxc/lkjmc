pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMMIT: &str = env!("LKJMC_BUILD_COMMIT");
pub const DIRTY_STATE: &str = env!("LKJMC_BUILD_DIRTY");

pub fn dirty() -> Option<bool> {
    match DIRTY_STATE {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn json() -> serde_json::Value {
    serde_json::json!({
        "version": VERSION,
        "commit": COMMIT,
        "dirty": dirty()
    })
}

pub fn dirty_label() -> &'static str {
    match dirty() {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_identity_is_not_a_placeholder() {
        assert_eq!(VERSION, "0.1.0-alpha.1");
        assert_ne!(VERSION, "0.0.0");
        assert!(COMMIT == "unknown" || valid_commit(COMMIT));
        assert!(matches!(DIRTY_STATE, "false" | "unknown"));
    }

    fn valid_commit(value: &str) -> bool {
        value.len() == 40
            && value
                .bytes()
                .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
    }
}
