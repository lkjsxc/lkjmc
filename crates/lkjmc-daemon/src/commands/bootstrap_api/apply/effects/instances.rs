use std::collections::BTreeMap;

use lkjmc_core::instance::InstanceKind;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::support::instance_helpers::store;

pub struct InstanceShape<'a> {
    pub kind: InstanceKind,
    pub server_port: u16,
    pub memory_mb: u32,
    pub bind_host: &'a str,
    pub public_hosts: &'a [String],
    pub backend_addresses: &'a BTreeMap<String, String>,
    pub forwarding_secret_file: &'a str,
    pub online_mode: bool,
    pub daemon_http_url: &'a str,
    pub _daemon_http_token_file: &'a str,
    pub eula_accepted: bool,
    pub server_asset_path: &'a str,
    pub server_asset_sha256: &'a str,
}

pub fn reconcile(
    client: &mut postgres::Client,
    id: &str,
    shape: InstanceShape<'_>,
) -> Result<(), String> {
    let jar = store(lkjmc_store::jar::get_by_path(
        client,
        shape.server_asset_path,
    ))?
    .ok_or_else(|| format!("configured server jar asset not found: {id}"))?;
    if !jar.sha256.eq_ignore_ascii_case(shape.server_asset_sha256) {
        return Err(format!("configured server jar digest differs: {id}"));
    }
    if jar.project != project(shape.kind) {
        return Err(format!("configured server jar project differs: {id}"));
    }
    let config = instance_config(id, &shape, jar.id)?;
    let exists = lkjmc_store::instance::get(client, id)
        .map_err(|error| error.to_string())?
        .is_some();
    if exists {
        store(lkjmc_store::instance::reserve_port(
            client,
            id,
            i32::from(shape.server_port),
            "server",
        ))?;
        store(lkjmc_store::instance::update_config(client, id, &config))?;
    } else {
        store(lkjmc_store::instance::insert(
            client,
            id,
            None,
            kind_text(shape.kind),
            "stopped",
            &config,
        ))?;
        if let Err(error) = store(lkjmc_store::instance::reserve_port(
            client,
            id,
            i32::from(shape.server_port),
            "server",
        )) {
            let _ = lkjmc_store::instance::delete(client, id);
            return Err(error);
        }
    }
    store(lkjmc_store::instance::set_jar_asset(client, id, jar.id))?;
    Ok(())
}

fn instance_config(id: &str, shape: &InstanceShape<'_>, jar_id: Uuid) -> Result<Value, String> {
    let mut config = json!({
        "template": template(shape.kind),
        "serverPort": shape.server_port,
        "memoryMb": shape.memory_mb,
        "jarAssetId": jar_id.to_string(),
        "forwardingSecretFile": shape.forwarding_secret_file,
        "proxyOnlineMode": shape.online_mode,
        "env": {
            "LKJMC_INSTANCE_ID": id,
            "LKJMC_DAEMON_HTTP_URL": shape.daemon_http_url,
            "LKJMC_SERVER_IMPLEMENTATION": kind_text(shape.kind)
        }
    });
    if shape.kind == InstanceKind::Velocity {
        config["bind"] = json!(format!("{}:{}", shape.bind_host, shape.server_port));
        config["backendAddresses"] = json!(shape.backend_addresses);
        config["publicHosts"] = json!(shape.public_hosts);
    } else {
        config["eulaAccepted"] = json!(shape.eula_accepted);
        config["velocityProxy"] = json!(true);
        config["properties"] = json!({
            "motd": format!("lkjmc {id}"),
            "gamemode": "survival",
            "server-ip": shape.bind_host
        });
    }
    Ok(config)
}

fn project(kind: InstanceKind) -> &'static str {
    match kind {
        InstanceKind::Velocity => "velocity",
        InstanceKind::Paper => "paper",
        InstanceKind::Folia => "folia",
        InstanceKind::Purpur => "purpur",
        InstanceKind::VanillaCustom | InstanceKind::ModdedCustom => "paper",
    }
}

fn template(kind: InstanceKind) -> &'static str {
    match kind {
        InstanceKind::Velocity => "velocity-modern",
        InstanceKind::Folia => "folia-survival",
        InstanceKind::Purpur => "purpur-survival",
        _ => "paper-survival",
    }
}

fn kind_text(kind: InstanceKind) -> &'static str {
    match kind {
        InstanceKind::Velocity => "velocity",
        InstanceKind::Paper => "paper",
        InstanceKind::Folia => "folia",
        InstanceKind::Purpur => "purpur",
        InstanceKind::VanillaCustom => "vanilla-custom",
        InstanceKind::ModdedCustom => "modded-custom",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rendered_instance_config_withholds_root_daemon_token() -> Result<(), String> {
        let secret = temp_secret("forwarding-secret")?;
        let hosts = vec!["play.example.test".to_string()];
        let backend_addresses = BTreeMap::new();
        let shape = InstanceShape {
            kind: InstanceKind::Folia,
            server_port: 25566,
            memory_mb: 2048,
            bind_host: "0.0.0.0",
            public_hosts: &hosts,
            backend_addresses: &backend_addresses,
            forwarding_secret_file: &secret,
            online_mode: true,
            daemon_http_url: "http://127.0.0.1:8765",
            _daemon_http_token_file: "/etc/lkjmc/daemon-http.token",
            eula_accepted: true,
            server_asset_path: "/tmp/folia.jar",
            server_asset_sha256: "f52c408490a0225611e67907a3ca19f7e6da2c6bc899e715d5f46844e7103c39",
        };
        let config = instance_config("hub", &shape, Uuid::nil())?;
        fs::remove_file(&secret).ok();
        assert_eq!(config["env"]["LKJMC_INSTANCE_ID"], json!("hub"));
        assert_eq!(
            config["env"]["LKJMC_DAEMON_HTTP_URL"],
            json!("http://127.0.0.1:8765")
        );
        assert!(config["env"].get("LKJMC_DAEMON_HTTP_TOKEN_FILE").is_none());
        assert_eq!(config["eulaAccepted"], json!(true));
        assert!(config.get("forwardingSecret").is_none());
        assert_eq!(config["forwardingSecretFile"], json!(secret));
        Ok(())
    }

    #[test]
    fn survival_backend_receives_the_accepted_eula_and_forwarding_shape() -> Result<(), String> {
        let backend_addresses = BTreeMap::new();
        let shape = InstanceShape {
            kind: InstanceKind::Folia,
            server_port: 25567,
            memory_mb: 2048,
            bind_host: "127.0.0.1",
            public_hosts: &[],
            backend_addresses: &backend_addresses,
            forwarding_secret_file: "/tmp/forwarding.secret",
            online_mode: true,
            daemon_http_url: "http://127.0.0.1:8765",
            _daemon_http_token_file: "/etc/lkjmc/daemon-http.token",
            eula_accepted: true,
            server_asset_path: "/tmp/folia.jar",
            server_asset_sha256: "f52c408490a0225611e67907a3ca19f7e6da2c6bc899e715d5f46844e7103c39",
        };
        let config = instance_config("survival", &shape, Uuid::nil())?;
        assert_eq!(config["eulaAccepted"], json!(true));
        assert_eq!(config["velocityProxy"], json!(true));
        assert_eq!(config["properties"]["motd"], json!("lkjmc survival"));
        assert_eq!(config["properties"]["server-ip"], json!("127.0.0.1"));
        Ok(())
    }

    #[test]
    fn rendered_proxy_config_carries_all_backend_addresses() -> Result<(), String> {
        let secret = temp_secret("forwarding-secret")?;
        let hosts = vec!["play.example.test".to_string()];
        let backend_addresses = BTreeMap::from([
            ("hub".to_string(), "127.0.0.1:25566".to_string()),
            ("survival".to_string(), "127.0.0.1:25567".to_string()),
        ]);
        let shape = InstanceShape {
            kind: InstanceKind::Velocity,
            server_port: 25565,
            memory_mb: 1024,
            bind_host: "0.0.0.0",
            public_hosts: &hosts,
            backend_addresses: &backend_addresses,
            forwarding_secret_file: &secret,
            online_mode: false,
            daemon_http_url: "http://127.0.0.1:8765",
            _daemon_http_token_file: "/etc/lkjmc/daemon-http.token",
            eula_accepted: true,
            server_asset_path: "/tmp/velocity.jar",
            server_asset_sha256: "fe53021f3168322cb6cb68f78699866fd098df3c306e4359847a10b0d02689ef",
        };
        let config = instance_config("proxy", &shape, Uuid::nil())?;
        fs::remove_file(secret).ok();
        assert_eq!(config["bind"], json!("0.0.0.0:25565"));
        assert_eq!(
            config["backendAddresses"],
            json!({"hub":"127.0.0.1:25566","survival":"127.0.0.1:25567"})
        );
        assert_eq!(config["publicHosts"], json!(["play.example.test"]));
        assert_eq!(config["proxyOnlineMode"], json!(false));
        Ok(())
    }

    fn temp_secret(contents: &str) -> Result<String, String> {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lkjmc-secret-{suffix}"));
        fs::write(&path, contents).map_err(|error| error.to_string())?;
        Ok(path.to_string_lossy().into_owned())
    }
}
