use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::app::AppState;

pub fn render_instance(
    state: &AppState,
    id: &str,
    kind: &str,
    config: &Value,
) -> Result<PathBuf, String> {
    if !lkjmc_core::validation::is_kebab_id(id) {
        return Err("invalid instance id".to_string());
    }
    let template = config
        .get("template")
        .and_then(Value::as_str)
        .unwrap_or("default");
    if !lkjmc_core::validation::is_kebab_id(template) {
        return Err("invalid template id".to_string());
    }
    let data_root = state.data_root();
    let dir = Path::new(&data_root).join(id);
    fs::create_dir_all(&dir).map_err(|error| format!("create instance dir: {error}"))?;
    let template = load_template(state, template)?;
    render_files(&dir, &template)?;
    render_files(&dir, config)?;
    match kind {
        "velocity" => render_velocity(&dir, config, &template)?,
        "paper" | "folia" | "vanilla-custom" | "modded-custom" => {
            render_server(&dir, config, &template)?;
        }
        other => return Err(format!("unsupported instance kind: {other}")),
    }
    Ok(dir)
}

fn load_template(state: &AppState, name: &str) -> Result<Value, String> {
    let config_root = state.config_root();
    let path = Path::new(&config_root)
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
            "properties": {"gamemode": "survival", "enable-command-block": false}
        }),
        "velocity-modern" => json!({"velocityForwardingMode": "modern"}),
        _ => json!({}),
    }
}

fn render_server(dir: &Path, config: &Value, template: &Value) -> Result<(), String> {
    write_file(&dir.join("eula.txt"), "eula=true\n")?;
    let mut properties = property_map(template.get("properties"))?;
    properties.extend(property_map(config.get("properties"))?);
    properties.insert("server-port".to_string(), port(config).to_string());
    properties
        .entry("motd".to_string())
        .or_insert_with(|| "lkjmc managed server".to_string());
    write_file(&dir.join("server.properties"), &property_file(&properties))
}

fn render_velocity(dir: &Path, config: &Value, template: &Value) -> Result<(), String> {
    let mode = config
        .get("velocityForwardingMode")
        .or_else(|| template.get("velocityForwardingMode"))
        .and_then(Value::as_str)
        .unwrap_or("modern");
    write_file(
        &dir.join("velocity.toml"),
        &format!(
            "bind = \"0.0.0.0:{}\"\nplayer-info-forwarding-mode = \"{mode}\"\n",
            port(config)
        ),
    )
}

fn render_files(dir: &Path, template: &Value) -> Result<(), String> {
    let Some(files) = template.get("files").and_then(Value::as_object) else {
        return Ok(());
    };
    for (name, content) in files {
        let content = content
            .as_str()
            .ok_or_else(|| format!("template file {name} must be a string"))?;
        let path = safe_child(dir, name)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create {}: {error}", parent.display()))?;
        }
        write_file(&path, content)?;
    }
    Ok(())
}

fn safe_child(root: &Path, name: &str) -> Result<PathBuf, String> {
    let path = Path::new(name);
    if path.is_absolute() || name.contains("..") {
        return Err(format!("unsafe template path: {name}"));
    }
    Ok(root.join(path))
}

fn property_map(value: Option<&Value>) -> Result<BTreeMap<String, String>, String> {
    let mut result = BTreeMap::new();
    let Some(map) = value.and_then(Value::as_object) else {
        return Ok(result);
    };
    for (key, value) in map {
        result.insert(key.clone(), property_value(value)?);
    }
    Ok(result)
}

fn property_value(value: &Value) -> Result<String, String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err("property values must be strings, booleans, or numbers".to_string()),
    }
}

fn property_file(properties: &BTreeMap<String, String>) -> String {
    properties
        .iter()
        .map(|(key, value)| format!("{key}={value}\n"))
        .collect()
}

fn port(config: &Value) -> i64 {
    config
        .get("serverPort")
        .and_then(Value::as_i64)
        .unwrap_or(25565)
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|error| format!("write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_filesystem_template() -> Result<(), String> {
        let root = std::env::temp_dir().join(format!("lkjmc-template-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("config/templates")).map_err(|error| error.to_string())?;
        fs::write(root.join("config/templates/custom.json"), template())
            .map_err(|error| error.to_string())?;
        let state = AppState::with_config_path(
            None,
            root.join("config").to_string_lossy().to_string(),
            root.join("logs").to_string_lossy().to_string(),
            root.join("jars").to_string_lossy().to_string(),
            root.join("data").to_string_lossy().to_string(),
            None,
        );
        let dir = render_instance(
            &state,
            "hub",
            "paper",
            &json!({"template":"custom","serverPort":25570}),
        )?;
        let props =
            fs::read_to_string(dir.join("server.properties")).map_err(|error| error.to_string())?;
        assert!(props.contains("difficulty=hard"));
        assert!(dir.join("paper-global.yml").exists());
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    fn template() -> &'static str {
        r#"{"properties":{"difficulty":"hard"},"files":{"paper-global.yml":"unsupported-settings: {}\n"}}"#
    }
}
