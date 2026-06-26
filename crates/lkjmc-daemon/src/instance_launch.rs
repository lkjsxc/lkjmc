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
