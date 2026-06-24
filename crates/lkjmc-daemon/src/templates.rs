use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

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
    let dir = Path::new(&state.data_root).join(id);
    fs::create_dir_all(&dir).map_err(|error| format!("create instance dir: {error}"))?;
    match kind {
        "velocity" => render_velocity(&dir, config)?,
        "paper" | "folia" | "vanilla-custom" | "modded-custom" => {
            render_server(&dir, config)?;
        }
        other => return Err(format!("unsupported instance kind: {other}")),
    }
    Ok(dir)
}

fn render_server(dir: &Path, config: &Value) -> Result<(), String> {
    write_file(&dir.join("eula.txt"), "eula=true\n")?;
    let port = config
        .get("serverPort")
        .and_then(Value::as_i64)
        .unwrap_or(25565);
    let motd = config
        .get("properties")
        .and_then(|value| value.get("motd"))
        .and_then(Value::as_str)
        .unwrap_or("lkjmc managed server");
    write_file(
        &dir.join("server.properties"),
        &format!("server-port={port}\nmotd={motd}\n"),
    )
}

fn render_velocity(dir: &Path, config: &Value) -> Result<(), String> {
    let port = config
        .get("serverPort")
        .and_then(Value::as_i64)
        .unwrap_or(25565);
    write_file(
        &dir.join("velocity.toml"),
        &format!("bind = \"0.0.0.0:{port}\"\nplayer-info-forwarding-mode = \"modern\"\n"),
    )
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|error| format!("write {}: {error}", path.display()))
}
