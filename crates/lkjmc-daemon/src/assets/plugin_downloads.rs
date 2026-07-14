mod geyser;
mod io;

use lkjmc_core::plugin::PluginId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;

pub fn sync(
    state: &AppState,
    client: &mut postgres::Client,
    plugin: PluginId,
) -> Result<lkjmc_store::asset::AssetRecord, String> {
    match plugin {
        PluginId::ViaVersion => sync_modrinth(state, client, plugin, "viaversion"),
        PluginId::ViaBackwards => sync_modrinth(state, client, plugin, "viabackwards"),
        PluginId::Geyser | PluginId::Floodgate => geyser::sync(state, client, plugin),
        other => Err(format!(
            "plugin download source not implemented for {}",
            other.as_str()
        )),
    }
}

fn sync_modrinth(
    state: &AppState,
    client: &mut postgres::Client,
    plugin: PluginId,
    slug: &str,
) -> Result<lkjmc_store::asset::AssetRecord, String> {
    let selected = select_modrinth_file(slug)?;
    let path = io::target_path(
        &state.asset_root(),
        "modrinth",
        plugin.as_str(),
        &selected.sha512,
        &selected.filename,
    )?;
    let path_text = path.to_string_lossy().to_string();
    let download = io::download_to(
        &selected.url,
        &path,
        selected.size_bytes,
        crate::assets::download_io::ExpectedChecksum::Sha512(&selected.sha512),
    );
    if let Err(error) = &download {
        record_download(client, None, plugin, &selected, "failed", Some(error))?;
    }
    let hashes = download?;
    if let Some(asset) =
        lkjmc_store::asset::get_by_path(client, &path_text).map_err(|error| error.to_string())?
    {
        return Ok(asset);
    }
    let asset = insert_asset(client, plugin, &selected, &path_text, &hashes.sha256)?;
    record_download(client, Some(asset.id), plugin, &selected, "succeeded", None)?;
    Ok(asset)
}

fn select_modrinth_file(slug: &str) -> Result<SelectedFile, String> {
    let url = format!("https://api.modrinth.com/v2/project/{slug}/version");
    let versions = ureq::get(&url)
        .set("User-Agent", crate::commands::downloads::USER_AGENT)
        .call()
        .map_err(|error| format!("Modrinth request failed: {error}"))?
        .into_json::<Value>()
        .map_err(|error| format!("Modrinth JSON failed: {error}"))?;
    let array = versions
        .as_array()
        .ok_or_else(|| "Modrinth versions response was not an array".to_string())?;
    for version in array {
        if let Some(file) = selected_file(version)? {
            return Ok(file);
        }
    }
    Err(format!("no verified Modrinth file found for {slug}"))
}

fn selected_file(version: &Value) -> Result<Option<SelectedFile>, String> {
    let Some(files) = version.get("files").and_then(Value::as_array) else {
        return Ok(None);
    };
    let file = files
        .iter()
        .find(|file| {
            file.get("primary")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .or_else(|| files.first());
    let Some(file) = file else {
        return Ok(None);
    };
    let filename = str_field(file, "filename")?;
    if !filename.ends_with(".jar") {
        return Ok(None);
    }
    let sha512 = file
        .get("hashes")
        .and_then(|hashes| hashes.get("sha512"))
        .and_then(Value::as_str)
        .ok_or_else(|| "Modrinth file missing sha512".to_string())?;
    Ok(Some(SelectedFile {
        filename: filename.to_string(),
        url: str_field(file, "url")?.to_string(),
        sha512: sha512.to_ascii_lowercase(),
        size_bytes: file
            .get("size")
            .and_then(Value::as_i64)
            .ok_or_else(|| "Modrinth file missing size".to_string())?,
    }))
}
fn insert_asset(
    client: &mut postgres::Client,
    plugin: PluginId,
    file: &SelectedFile,
    path: &str,
    sha256: &str,
) -> Result<lkjmc_store::asset::AssetRecord, String> {
    let id = Uuid::new_v4();
    if let Err(error) = lkjmc_store::asset::insert(
        client,
        lkjmc_store::asset::NewAsset {
            id,
            asset_kind: "plugin",
            platform: "paper",
            project: plugin.as_str(),
            channel: "stable",
            name: plugin.as_str(),
            file_name: &file.filename,
            path,
            sha256,
            size_bytes: file.size_bytes,
            source: "modrinth",
            metadata: json!({"sha512": file.sha512}),
        },
    ) {
        return lkjmc_store::asset::get_by_path(client, path)
            .map_err(|lookup| lookup.to_string())?
            .ok_or_else(|| error.to_string());
    }
    lkjmc_store::asset::get(client, id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "inserted plugin asset missing".to_string())
}

fn record_download(
    client: &mut postgres::Client,
    asset_id: Option<Uuid>,
    plugin: PluginId,
    file: &SelectedFile,
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
            sha256: None,
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

struct SelectedFile {
    filename: String,
    url: String,
    sha512: String,
    size_bytes: i64,
}
