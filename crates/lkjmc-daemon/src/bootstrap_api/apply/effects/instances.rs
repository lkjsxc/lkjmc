use lkjmc_core::instance::InstanceKind;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::instance_helpers::store;

pub struct InstanceShape<'a> {
    pub kind: InstanceKind,
    pub server_port: u16,
    pub memory_mb: u32,
    pub bind_host: &'a str,
    pub public_hosts: &'a [String],
    pub backend_address: Option<&'a str>,
    pub forwarding_secret_file: &'a str,
    pub daemon_http_url: &'a str,
    pub daemon_http_token_file: &'a str,
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
    if lkjmc_store::instance::get(client, id)
        .map_err(|error| error.to_string())?
        .is_some()
    {
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
    }
    let _ = lkjmc_store::instance::reserve_port(client, id, i32::from(shape.server_port), "server");
    store(lkjmc_store::instance::set_jar_asset(client, id, jar.id))?;
    Ok(())
}

fn instance_config(id: &str, shape: &InstanceShape<'_>, jar_id: Uuid) -> Result<Value, String> {
    let secret = super::secrets::read_secret(shape.forwarding_secret_file)?;
    let mut config = json!({
        "template": template(shape.kind),
        "serverPort": shape.server_port,
        "memoryMb": shape.memory_mb,
        "jarAssetId": jar_id.to_string(),
        "forwardingSecret": secret,
        "proxyOnlineMode": true,
        "env": {
            "LKJMC_INSTANCE_ID": id,
            "LKJMC_DAEMON_HTTP_URL": shape.daemon_http_url,
            "LKJMC_DAEMON_HTTP_TOKEN_FILE": shape.daemon_http_token_file,
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
