mod velocity_hosts;
use crate::app::AppState;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
pub fn render_instance(
    state: &AppState,
    id: &str,
    kind: &str,
    config: &Value,
) -> Result<PathBuf, String> {
    validate_id(id, "instance")?;
    let template_name = config
        .get("template")
        .and_then(Value::as_str)
        .unwrap_or("default");
    validate_id(template_name, "template")?;
    let dir = Path::new(&state.data_root()).join(id);
    fs::create_dir_all(&dir).map_err(|error| format!("create instance dir: {error}"))?;
    let template = load_template(state, template_name)?;
    render_files(&dir, &template)?;
    render_files(&dir, config)?;
    fs::create_dir_all(dir.join("plugins")).map_err(|error| format!("create plugins: {error}"))?;
    match kind {
        "velocity" => render_velocity(&dir, config, &template),
        "paper" | "folia" | "vanilla-custom" | "modded-custom" => {
            render_server(&dir, config, &template)
        }
        other => Err(format!("unsupported instance kind: {other}")),
    }?;
    Ok(dir)
}

fn load_template(state: &AppState, name: &str) -> Result<Value, String> {
    let path = Path::new(&state.config_root())
        .join("templates")
        .join(format!("{name}.json"));
    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|error| format!("read template {}: {error}", path.display()))?;
        serde_json::from_str(&content).map_err(|error| format!("parse template {name}: {error}"))
    } else {
        Ok(builtin_template(name))
    }
}

fn builtin_template(name: &str) -> Value {
    match name {
        "paper-survival" | "folia-survival" => json!({
            "properties": {"gamemode": "survival", "enable-command-block": false},
            "velocityProxy": true
        }),
        "velocity-modern" => json!({"velocityForwardingMode": "modern"}),
        _ => json!({}),
    }
}

fn render_server(dir: &Path, config: &Value, template: &Value) -> Result<(), String> {
    let eula = if bool_value(config, "eulaAccepted") {
        "true"
    } else {
        "false"
    };
    write_file(&dir.join("eula.txt"), &format!("eula={eula}\n"))?;
    let mut properties = property_map(template.get("properties"))?;
    properties.extend(property_map(config.get("properties"))?);
    properties.insert("server-port".to_string(), port(config, 25566).to_string());
    properties
        .entry("motd".to_string())
        .or_insert_with(|| "lkjmc hub".to_string());
    if bool_value(template, "velocityProxy") || bool_value(config, "velocityProxy") {
        properties.insert("online-mode".to_string(), "false".to_string());
    }
    write_file(&dir.join("server.properties"), &property_file(&properties))?;
    write_file(&dir.join("spigot.yml"), "settings:\n  bungeecord: false\n")?;
    let secret = config
        .get("forwardingSecret")
        .and_then(Value::as_str)
        .unwrap_or("");
    let online = config
        .get("proxyOnlineMode")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    fs::create_dir_all(dir.join("config")).map_err(|error| format!("create config: {error}"))?;
    write_file(
        &dir.join("config/paper-global.yml"),
        &format!(
            "proxies:\n  velocity:\n    enabled: true\n    online-mode: {online}\n    secret: \"{}\"\n",
            secret.replace('"', "\\\"")
        ),
    )
}

fn render_velocity(dir: &Path, config: &Value, template: &Value) -> Result<(), String> {
    let mode = config
        .get("velocityForwardingMode")
        .or_else(|| template.get("velocityForwardingMode"))
        .and_then(Value::as_str)
        .unwrap_or("modern");
    let bind = config
        .get("bind")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("0.0.0.0:{}", port(config, 25565)));
    let hub = config
        .get("hubAddress")
        .and_then(Value::as_str)
        .unwrap_or("127.0.0.1:25566");
    let secret = config
        .get("forwardingSecret")
        .and_then(Value::as_str)
        .unwrap_or("");
    let forced_hosts = velocity_hosts::forced_hosts(config, "hub");
    write_file(&dir.join("forwarding.secret"), secret)?;
    write_file(
        &dir.join("velocity.toml"),
        &format!(
            "config-version = \"2.7\"\nbind = \"{bind}\"\nmotd = \"lkjmc network\"\nshow-max-players = 20\nonline-mode = true\nforce-key-authentication = true\nplayer-info-forwarding-mode = \"{mode}\"\nforwarding-secret-file = \"forwarding.secret\"\nping-passthrough = \"disabled\"\n\n[servers]\nhub = \"{hub}\"\n\ntry = [\"hub\"]\n\n[forced-hosts]\n{forced_hosts}"
        ),
    )
}

fn render_files(dir: &Path, config: &Value) -> Result<(), String> {
    let Some(files) = config.get("files").and_then(Value::as_object) else {
        return Ok(());
    };
    for (relative, content) in files {
        let path = safe_child(dir, relative)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create {}: {error}", parent.display()))?;
        }
        write_file(&path, content.as_str().unwrap_or_default())?;
    }
    Ok(())
}

fn property_map(value: Option<&Value>) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    let Some(object) = value.and_then(Value::as_object) else {
        return Ok(map);
    };
    for (key, value) in object {
        map.insert(key.clone(), scalar(value)?);
    }
    Ok(map)
}

fn property_file(properties: &BTreeMap<String, String>) -> String {
    properties
        .iter()
        .map(|(key, value)| format!("{key}={value}\n"))
        .collect::<String>()
}

fn scalar(value: &Value) -> Result<String, String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err("template property values must be scalar".to_string()),
    }
}

fn port(config: &Value, fallback: i64) -> i64 {
    config
        .get("serverPort")
        .and_then(Value::as_i64)
        .unwrap_or(fallback)
}

fn bool_value(config: &Value, key: &str) -> bool {
    config.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn safe_child(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if path.is_absolute() || relative.contains("..") {
        return Err(format!("unsafe template path: {relative}"));
    }
    Ok(root.join(path))
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|error| format!("write {}: {error}", path.display()))
}

fn validate_id(value: &str, label: &str) -> Result<(), String> {
    if lkjmc_core::validation::is_kebab_id(value) {
        Ok(())
    } else {
        Err(format!("invalid {label} id"))
    }
}

#[cfg(test)]
mod tests;
