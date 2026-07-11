use std::fs;
use std::path::Path;

use serde_json::Value;

pub(super) fn forwarding(config: &Value) -> Result<String, String> {
    if config.get("forwardingSecret").is_some() {
        return Err("instance config must use forwardingSecretFile".to_string());
    }
    let path = config
        .get("forwardingSecretFile")
        .and_then(Value::as_str)
        .ok_or_else(|| "instance forwarding secret file is missing".to_string())?;
    let secret =
        fs::read_to_string(path).map_err(|error| format!("read forwarding secret: {error}"))?;
    let secret = secret.trim_end();
    if secret.is_empty() {
        return Err("forwarding secret is empty".to_string());
    }
    Ok(secret.to_string())
}

pub(super) fn write(path: &Path, content: &str) -> Result<(), String> {
    crate::support::private_file::replace_private(path, content.as_bytes())
}
