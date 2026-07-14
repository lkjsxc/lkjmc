use lkjmc_core::bootstrap::{BootstrapProfile, BootstrapRequest, BootstrapRuntimeSettings};
use lkjmc_core::config::{BedrockEntry, BedrockMode, JavaEntry, LkjmcConfig};
use serde_json::Value;

use crate::app::AppState;

pub(super) fn from_body(
    state: &AppState,
    body: &Value,
    dry_run: bool,
) -> Result<BootstrapRequest, String> {
    let profile = body
        .get("profile")
        .and_then(Value::as_str)
        .unwrap_or("playable");
    if profile != "playable" {
        return Err(format!("unsupported bootstrap profile: {profile}"));
    }
    let config = state.runtime_config()?;
    let mut java_entry = config
        .as_ref()
        .map(|config| config.network.java_entry())
        .unwrap_or_default();
    let mut bedrock_entry = config
        .as_ref()
        .map(|config| config.network.bedrock_entry())
        .unwrap_or_default();
    let plugin_policy = config
        .as_ref()
        .map(|config| config.plugins.clone())
        .unwrap_or_default();
    let online_mode = config
        .as_ref()
        .map(|config| config.network.online_mode())
        .unwrap_or(true);
    let runtime = config.as_ref().map(runtime_settings).unwrap_or_default();
    merge_java(body, &mut java_entry)?;
    merge_bedrock(body, &mut bedrock_entry)?;
    Ok(BootstrapRequest {
        profile: BootstrapProfile::Playable,
        accept_minecraft_eula: body
            .get("acceptMinecraftEula")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        online_mode,
        java_entry,
        bedrock_entry,
        plugin_policy,
        runtime,
        dry_run,
    })
}

fn runtime_settings(config: &LkjmcConfig) -> BootstrapRuntimeSettings {
    BootstrapRuntimeSettings {
        proxy_memory_mb: config.runtime.proxy_java_memory_mb,
        backend_memory_mb: config.runtime.default_java_memory_mb,
        forwarding_secret_file: config.network.forwarding_secret_file().to_string(),
        daemon_http_enabled: config.daemon_http.enabled,
        daemon_http_address: http_url(&config.daemon_http.address),
        daemon_http_token_file: config.daemon_http.token_file.clone(),
    }
}

fn http_url(address: &str) -> String {
    if address.starts_with("http://") || address.starts_with("https://") {
        address.to_string()
    } else {
        format!("http://{address}")
    }
}

fn merge_java(body: &Value, entry: &mut JavaEntry) -> Result<(), String> {
    if let Some(bind_host) = body.get("javaBindHost").and_then(Value::as_str) {
        if bind_host.is_empty() {
            return Err("javaBindHost must not be empty".to_string());
        }
        entry.bind_host = bind_host.to_string();
    }
    if let Some(port) = body.get("javaPort") {
        entry.port = as_port(port, "javaPort")?;
    }
    if let Some(host) = body.get("javaPublicHost").and_then(Value::as_str) {
        if host.is_empty() {
            return Err("javaPublicHost must not be empty".to_string());
        }
        if !entry.public_hosts.iter().any(|value| value == host) {
            entry.public_hosts.push(host.to_string());
        }
        entry.preferred_public_host = Some(host.to_string());
    }
    Ok(())
}

fn merge_bedrock(body: &Value, entry: &mut BedrockEntry) -> Result<(), String> {
    if let Some(mode) = body.get("bedrock").and_then(Value::as_str) {
        entry.mode = match mode {
            "auto" => BedrockMode::Auto,
            "enabled" => BedrockMode::Enabled,
            "disabled" => BedrockMode::Disabled,
            other => return Err(format!("unsupported bedrock mode: {other}")),
        };
    }
    if let Some(port) = body.get("bedrockPort") {
        entry.port = as_port(port, "bedrockPort")?;
    }
    Ok(())
}

fn as_port(value: &Value, field: &str) -> Result<u16, String> {
    let Some(port) = value.as_u64() else {
        return Err(format!("{field} must be an integer port"));
    };
    if port == 0 || port > u64::from(u16::MAX) {
        return Err(format!("{field} must be between 1 and 65535"));
    }
    Ok(port as u16)
}
