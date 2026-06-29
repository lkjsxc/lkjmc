use std::collections::BTreeMap;

use postgres::Client;
use serde_json::Value;
use uuid::Uuid;

use crate::app::AppState;
use crate::instance_helpers::body_string;

pub struct LaunchSpec {
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

pub fn launch(
    _state: &AppState,
    client: &mut Client,
    config: &Value,
) -> Result<LaunchSpec, String> {
    let env = env_map(config);
    if let Some(asset_id) = config.get("jarAssetId").and_then(Value::as_str) {
        let asset_id = Uuid::parse_str(asset_id).map_err(|error| error.to_string())?;
        let memory_mb = config
            .get("memoryMb")
            .and_then(Value::as_i64)
            .unwrap_or(2048);
        let (command, args) = crate::jars::verified_launch(client, asset_id, memory_mb)?;
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

fn env_map(config: &Value) -> BTreeMap<String, String> {
    config
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
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::env_map;
    use serde_json::json;

    #[test]
    fn launch_environment_preserves_daemon_token_file_path() {
        let env = env_map(&json!({
            "env": {
                "LKJMC_INSTANCE_ID": "hub",
                "LKJMC_DAEMON_HTTP_URL": "http://127.0.0.1:8765",
                "LKJMC_DAEMON_HTTP_TOKEN_FILE": "/etc/lkjmc/daemon-http.token"
            }
        }));
        assert_eq!(
            env.get("LKJMC_INSTANCE_ID").map(String::as_str),
            Some("hub")
        );
        assert_eq!(
            env.get("LKJMC_DAEMON_HTTP_URL").map(String::as_str),
            Some("http://127.0.0.1:8765")
        );
        assert_eq!(
            env.get("LKJMC_DAEMON_HTTP_TOKEN_FILE").map(String::as_str),
            Some("/etc/lkjmc/daemon-http.token")
        );
    }
}
