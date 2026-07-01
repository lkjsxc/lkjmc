use serde_json::{json, Value};
use uuid::Uuid;

use crate::config::Config;

pub fn send(config: &Config, command: &str, body: Value) -> Result<Value, String> {
    let token = config.daemon_secret()?;
    let payload = json!({
        "requestId": Uuid::new_v4().to_string(),
        "actor": {"kind":"daemon", "name": config.audit_actor},
        "command": command,
        "body": body
    });
    let response = ureq::post(&config.daemon_http_url)
        .set("authorization", &format!("Bearer {token}"))
        .send_json(payload)
        .map_err(|error| error.to_string())?;
    response.into_json().map_err(|error| error.to_string())
}

pub fn status(config: &Config) -> Result<String, String> {
    let value = send(
        config,
        "status",
        json!({"principalKind":"discord-user", "principalId":"startup-check"}),
    )?;
    Ok(value
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        .to_string())
}
