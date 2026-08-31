use std::fs;
use std::path::Path;

use lkjmc_core::temporary::{TemporaryInstancePlan, TemporaryRuntimeFacts};
use serde_json::{json, Value};

use crate::app::AppState;

pub fn runtime_facts(
    state: &AppState,
    client: &mut postgres::Client,
) -> Result<TemporaryRuntimeFacts, String> {
    let rows = client
        .query("select port from instance_ports order by port", &[])
        .map_err(|error| error.to_string())?;
    let config = state.runtime_config()?.map(|config| config.runtime);
    Ok(TemporaryRuntimeFacts {
        occupied_ports: rows
            .into_iter()
            .filter_map(|row| u16::try_from(row.get::<_, i32>(0)).ok())
            .collect(),
        port_range_start: config
            .as_ref()
            .map(|value| value.port_range_start)
            .unwrap_or(25566),
        port_range_end: config
            .as_ref()
            .map(|value| value.port_range_end)
            .unwrap_or(25665),
    })
}

pub fn instance_config(
    state: &AppState,
    plan: &TemporaryInstancePlan,
    jar_id: &str,
) -> Result<Value, String> {
    let config = state.runtime_config()?;
    let runtime = config.as_ref().map(|config| &config.runtime);
    let daemon = config.as_ref().map(|config| &config.daemon_http);
    Ok(json!({
        "template": "folia-survival",
        "serverPort": plan.server_port,
        "memoryMb": runtime.map(|value| value.default_java_memory_mb).unwrap_or(2048),
        "jarAssetId": jar_id,
        "velocityProxy": true,
        "forwardingSecretFile": forwarding_secret_file(state)?,
        "proxyOnlineMode": true,
        "properties": {"motd": "temporary adventure", "level-name": plan.world_path},
        "env": {
            "LKJMC_INSTANCE_ID": plan.instance_id,
            "LKJMC_DAEMON_HTTP_URL": daemon.map(|value| http_url(&value.address)).unwrap_or_else(|| "http://127.0.0.1:8765".to_string()),
            "LKJMC_SERVER_IMPLEMENTATION": "folia"
        }
    }))
}

pub fn ensure_new_world(path: &str) -> Result<(), String> {
    if Path::new(path).exists() {
        return Err(format!("world path already exists: {path}"));
    }
    Ok(())
}

pub fn forwarding_secret_file(state: &AppState) -> Result<String, String> {
    let path = state
        .runtime_config()?
        .map(|config| config.network.forwarding_secret_file().to_string())
        .unwrap_or_else(|| "/etc/lkjmc/forwarding.secret".to_string());
    let secret =
        fs::read_to_string(&path).map_err(|error| format!("read forwarding secret: {error}"))?;
    if secret.trim().is_empty() {
        Err("forwarding secret is empty".to_string())
    } else {
        Ok(path)
    }
}

fn http_url(address: &str) -> String {
    if address.starts_with("http://") || address.starts_with("https://") {
        address.to_string()
    } else {
        format!("http://{address}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lkjmc_core::temporary::{CleanupPolicy, TemporaryInstancePlan};

    #[test]
    fn temporary_config_withholds_root_token_path() -> Result<(), String> {
        let root = std::env::temp_dir().join(format!("temporary-config-{}", std::process::id()));
        let secret = root.join("forwarding.secret");
        let config = root.join("daemon.json");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        fs::write(&secret, "test-secret").map_err(|error| error.to_string())?;
        let template = include_str!("../../../../../config/defaults/daemon.json.example");
        fs::write(
            &config,
            template.replace("/etc/lkjmc/forwarding.secret", &secret.to_string_lossy()),
        )
        .map_err(|error| error.to_string())?;
        let state = AppState::with_config_path(
            None,
            8,
            "/c".into(),
            "/l".into(),
            "/j".into(),
            "/d".into(),
            Some(config.to_string_lossy().to_string()),
            None,
            None,
        );
        let plan = TemporaryInstancePlan {
            instance_id: "temp".into(),
            server_port: 25566,
            world_path: "/d/temp".into(),
            visibility: "private".into(),
            max_lifetime_seconds: 60,
            retention_seconds: 60,
            cleanup_policy: CleanupPolicy::Delete,
        };
        let config = instance_config(&state, &plan, "jar")?;
        assert!(config["env"].get("LKJMC_DAEMON_HTTP_TOKEN_FILE").is_none());
        Ok(())
    }
}
