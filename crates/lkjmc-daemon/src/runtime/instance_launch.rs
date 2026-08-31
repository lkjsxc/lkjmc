use std::collections::BTreeMap;

use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::app::AppState;
use crate::support::instance_helpers::body_string;

pub struct LaunchSpec {
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

pub fn launch(
    _state: &AppState,
    client: &mut Client,
    kind: &str,
    config: &Value,
) -> Result<LaunchSpec, String> {
    let env = env_map(kind, config);
    if let Some(asset_id) = config.get("jarAssetId").and_then(Value::as_str) {
        let asset_id = Uuid::parse_str(asset_id).map_err(|error| error.to_string())?;
        let memory_mb = config
            .get("memoryMb")
            .and_then(Value::as_i64)
            .unwrap_or(2048);
        let (command, args) = crate::commands::jars::verified_launch(client, asset_id, memory_mb)?;
        return Ok(LaunchSpec { command, args, env });
    }
    let launch = config
        .get("launch")
        .ok_or_else(|| "instance has no launch profile".to_string())?;
    let command = body_string(launch, "command")?;
    let args = launch
        .get("args")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    Ok(LaunchSpec { command, args, env })
}

fn env_map(kind: &str, config: &Value) -> BTreeMap<String, String> {
    let mut env: BTreeMap<String, String> = config
        .get("env")
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|text| (key.clone(), text.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();
    if let Some(port) = config.get("serverPort").and_then(Value::as_i64) {
        env.entry("LKJMC_SERVER_PORT".to_string())
            .or_insert_with(|| port.to_string());
    }
    env.entry("LKJMC_INSTANCE_KIND".to_string())
        .or_insert_with(|| kind.to_string());
    env
}

#[cfg(test)]
mod tests {
    use super::env_map;
    use serde_json::json;

    #[test]
    fn launch_environment_contains_only_instance_scoped_configuration() {
        let env = env_map(
            "paper",
            &json!({
                "env": {
                    "LKJMC_INSTANCE_ID": "quartz-world",
                    "LKJMC_HEARTBEAT_ENDPOINT": "http://127.0.0.1:8765/plugin/v1/heartbeat",
                    "LKJMC_HEARTBEAT_CREDENTIAL_FILE": "/var/lib/lkjmc/private/plugin-credentials/quartz-world.secret"
                },
                "serverPort": 25577
            }),
        );
        assert_eq!(
            env.get("LKJMC_INSTANCE_ID").map(String::as_str),
            Some("quartz-world")
        );
        assert_eq!(
            env.get("LKJMC_HEARTBEAT_ENDPOINT").map(String::as_str),
            Some("http://127.0.0.1:8765/plugin/v1/heartbeat")
        );
        assert_eq!(
            env.get("LKJMC_HEARTBEAT_CREDENTIAL_FILE")
                .map(String::as_str),
            Some("/var/lib/lkjmc/private/plugin-credentials/quartz-world.secret")
        );
        assert_eq!(
            env.get("LKJMC_SERVER_PORT").map(String::as_str),
            Some("25577")
        );
        assert_eq!(
            env.get("LKJMC_INSTANCE_KIND").map(String::as_str),
            Some("paper")
        );
    }
}
