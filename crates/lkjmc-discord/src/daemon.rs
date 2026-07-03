use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::config::Config;

pub fn send(config: &Config, command: &str, body: Value) -> Result<Value, String> {
    let token = config.daemon_secret()?;
    let payload = CommandEnvelope {
        request_id: CommandId::parse("request id", Uuid::new_v4().to_string())
            .map_err(|error| error.to_string())?,
        actor: Actor {
            kind: ActorKind::Daemon,
            name: config.audit_actor.clone(),
        },
        command: command.to_string(),
        body,
    };
    let response = ureq::post(&command_url(&config.daemon_http_url))
        .set("authorization", &format!("Bearer {token}"))
        .send_json(serde_json::to_value(payload).map_err(|error| error.to_string())?)
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

fn command_url(base: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if trimmed.ends_with("/command") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/command")
    }
}
