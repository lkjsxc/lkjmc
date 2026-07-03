use std::fs;

use lkjmc_core::bootstrap::PluginId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;

pub fn sync(
    state: &AppState,
    client: &mut postgres::Client,
    plugin: PluginId,
) -> Result<lkjmc_store::asset::AssetRecord, String> {
    let file = select_file(plugin)?;
    let path = super::io::target_path(
        &state.asset_root(),
        "geysermc",
        plugin.as_str(),
        &file.sha256,
        &file.filename,
    )?;
    let path_text = path.to_string_lossy().to_string();
    if let Some(asset) =
        lkjmc_store::asset::get_by_path(client, &path_text).map_err(|error| error.to_string())?
    {
        return Ok(asset);
    }
    fs::create_dir_all(super::io::parent(&path)?)
        .map_err(|error| format!("create asset dir: {error}"))?;
    let download = super::io::download_to(&file.url, &path, file.size_bytes);
    if let Err(error) = &download {
        record_download(client, None, plugin, &file, "failed", Some(error))?;
    }
    let hashes = download?;
    if hashes.sha256 != file.sha256 {
        let _ = fs::remove_file(&path);
        let error = "download checksum mismatch".to_string();
        record_download(client, None, plugin, &file, "failed", Some(&error))?;
        return Err(error);
    }
    let asset = insert_asset(client, plugin, &file, &path_text, &hashes.sha256)?;
    record_download(client, Some(asset.id), plugin, &file, "succeeded", None)?;
    Ok(asset)
}

fn select_file(plugin: PluginId) -> Result<GeyserFile, String> {
    let (project, download) = match plugin {
        PluginId::Geyser => ("geyser", "velocity"),
        PluginId::Floodgate => ("floodgate", "velocity"),
        other => return Err(format!("not a GeyserMC plugin: {}", other.as_str())),
    };
    let meta_url = format!(
        "https://download.geysermc.org/v2/projects/{project}/versions/latest/builds/latest"
    );
    let body = ureq::get(&meta_url)
        .set("User-Agent", crate::commands::downloads::USER_AGENT)
        .call()
        .map_err(|error| format!("GeyserMC request failed: {error}"))?
        .into_json::<Value>()
        .map_err(|error| format!("GeyserMC JSON failed: {error}"))?;
    let info = body
        .get("downloads")
        .and_then(|value| value.get(download))
        .ok_or_else(|| format!("GeyserMC response missing {download} download"))?;
    let sha256 = str_field(info, "sha256")?.to_ascii_lowercase();
    let filename = info
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_else(|| default_file(plugin))
        .to_string();
    let url = format!(
        "https://download.geysermc.org/v2/projects/{project}/versions/latest/builds/latest/downloads/{download}"
    );
    Ok(GeyserFile {
        filename,
        url: url.clone(),
        sha256,
        size_bytes: content_length(&url)?,
    })
}

fn content_length(url: &str) -> Result<i64, String> {
    let response = ureq::head(url)
        .set("User-Agent", crate::commands::downloads::USER_AGENT)
        .call()
        .map_err(|error| format!("GeyserMC size request failed: {error}"))?;
    response
        .header("content-length")
        .ok_or_else(|| "GeyserMC download missing content-length".to_string())?
        .parse::<i64>()
        .map_err(|error| format!("invalid content-length: {error}"))
}

fn insert_asset(
    client: &mut postgres::Client,
    plugin: PluginId,
    file: &GeyserFile,
    path: &str,
    sha256: &str,
) -> Result<lkjmc_store::asset::AssetRecord, String> {
    let id = Uuid::new_v4();
    lkjmc_store::asset::insert(
        client,
        lkjmc_store::asset::NewAsset {
            id,
            asset_kind: "plugin",
            platform: "velocity",
            project: plugin.as_str(),
            channel: "stable",
            name: plugin.as_str(),
            file_name: &file.filename,
            path,
            sha256,
            size_bytes: file.size_bytes,
            source: "geysermc",
            metadata: json!({}),
        },
    )
    .map_err(|error| error.to_string())?;
    lkjmc_store::asset::get(client, id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "inserted plugin asset missing".to_string())
}

fn record_download(
    client: &mut postgres::Client,
    asset_id: Option<Uuid>,
    plugin: PluginId,
    file: &GeyserFile,
    result: &str,
    error: Option<&str>,
) -> Result<(), String> {
    lkjmc_store::asset::insert_download(
        client,
        lkjmc_store::asset::NewAssetDownload {
            id: Uuid::new_v4(),
            asset_id,
            asset_kind: "plugin",
            project: plugin.as_str(),
            channel: "stable",
            url: &file.url,
            result,
            sha256: Some(&file.sha256),
            size_bytes: Some(file.size_bytes),
            error,
        },
    )
    .map_err(|error| error.to_string())
}

fn str_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing field: {field}"))
}

fn default_file(plugin: PluginId) -> &'static str {
    match plugin {
        PluginId::Geyser => "Geyser-Velocity.jar",
        PluginId::Floodgate => "floodgate-velocity.jar",
        _ => "plugin.jar",
    }
}

struct GeyserFile {
    filename: String,
    url: String,
    sha256: String,
    size_bytes: i64,
}
