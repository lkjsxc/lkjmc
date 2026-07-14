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
    pub backend_address: Option<&'a str>,
    pub forwarding_secret_file: &'a str,
    pub online_mode: bool,
    pub daemon_http_url: &'a str,
    pub _daemon_http_token_file: &'a str,
}

pub fn reconcile(
    client: &mut postgres::Client,
    id: &str,
    shape: InstanceShape<'_>,
) -> Result<(), String> {
    let jar = store(lkjmc_store::jar::latest_matching(
        client,
        project(shape.kind),
    ))?
    .ok_or_else(|| format!("server jar asset not found for {}", kind_text(shape.kind)))?;
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
    if id == "hub" {
        config["eulaAccepted"] = json!(true);
        config["velocityProxy"] = json!(true);
        config["properties"] = json!({"motd":"lkjmc hub", "gamemode":"survival"});
    }
    if id == "proxy" {
        config["bind"] = json!(format!("{}:{}", shape.bind_host, shape.server_port));
        config["hubAddress"] = json!(shape.backend_address.unwrap_or("127.0.0.1:25566"));
        config["publicHosts"] = json!(shape.public_hosts);
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
        let shape = InstanceShape {
            kind: InstanceKind::Folia,
            server_port: 25566,
            memory_mb: 2048,
            bind_host: "0.0.0.0",
            public_hosts: &hosts,
            backend_address: None,
            forwarding_secret_file: &secret,
            online_mode: true,
            daemon_http_url: "http://127.0.0.1:8765",
            _daemon_http_token_file: "/etc/lkjmc/daemon-http.token",
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
    fn rendered_proxy_config_carries_public_hosts_and_backend_address() -> Result<(), String> {
        let secret = temp_secret("forwarding-secret")?;
        let hosts = vec!["play.example.test".to_string()];
        let shape = InstanceShape {
            kind: InstanceKind::Velocity,
            server_port: 25565,
            memory_mb: 1024,
            bind_host: "0.0.0.0",
            public_hosts: &hosts,
            backend_address: Some("127.0.0.1:25566"),
            forwarding_secret_file: &secret,
            online_mode: false,
            daemon_http_url: "http://127.0.0.1:8765",
            _daemon_http_token_file: "/etc/lkjmc/daemon-http.token",
        };
        let config = instance_config("proxy", &shape, Uuid::nil())?;
        fs::remove_file(secret).ok();
        assert_eq!(config["bind"], json!("0.0.0.0:25565"));
        assert_eq!(config["hubAddress"], json!("127.0.0.1:25566"));
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
