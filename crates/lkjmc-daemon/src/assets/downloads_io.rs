use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::app::AppState;
use crate::commands::downloads::BuildInfo;

pub fn download_asset(
    state: &AppState,
    client: &mut postgres::Client,
    project: &str,
    channel: &str,
    build: &BuildInfo,
) -> Result<lkjmc_store::jar::JarAssetRecord, String> {
    let jar_root = state.jar_root();
    let target = target_path(&jar_root, project, &build.sha256, &build.name)?;
    let target_text = target.to_string_lossy().to_string();
    crate::assets::download_io::download(
        &build.url,
        &target,
        Some(build.size_bytes),
        crate::assets::download_io::ExpectedChecksum::Sha256(&build.sha256),
    )?;
    if let Some(asset) =
        lkjmc_store::jar::get_by_path(client, &target_text).map_err(|error| error.to_string())?
    {
        return Ok(asset);
    }
    insert_asset(client, project, channel, build, &target_text)
}

fn insert_asset(
    client: &mut postgres::Client,
    project: &str,
    channel: &str,
    build: &BuildInfo,
    target_text: &str,
) -> Result<lkjmc_store::jar::JarAssetRecord, String> {
    let id = Uuid::new_v4();
    if let Err(error) = lkjmc_store::jar::insert(
        client,
        lkjmc_store::jar::NewJarAsset {
            id,
            kind: project,
            project,
            channel: &channel.to_ascii_lowercase(),
            name: &build.name,
            path: target_text,
            sha256: &build.sha256,
            size_bytes: build.size_bytes,
            source: "papermc",
        },
    ) {
        return lkjmc_store::jar::get_by_path(client, target_text)
            .map_err(|lookup| lookup.to_string())?
            .ok_or_else(|| error.to_string());
    }
    record_download(client, id, build)?;
    lkjmc_store::jar::get(client, id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "inserted jar asset missing".to_string())
}

fn record_download(
    client: &mut postgres::Client,
    asset_id: Uuid,
    build: &BuildInfo,
) -> Result<(), String> {
    lkjmc_store::jar::insert_download(
        client,
        lkjmc_store::jar::NewJarDownload {
            id: Uuid::new_v4(),
            jar_asset_id: Some(asset_id),
            project: &build.project,
            channel: &build.channel,
            url: &build.url,
            result: "succeeded",
            sha256: Some(&build.sha256),
            size_bytes: Some(build.size_bytes),
        },
    )
    .map_err(|error| error.to_string())
}

fn target_path(root: &str, project: &str, sha256: &str, name: &str) -> Result<PathBuf, String> {
    let short = sha256
        .get(0..12)
        .ok_or_else(|| "invalid sha256".to_string())?;
    Ok(Path::new(root)
        .join("papermc")
        .join(project)
        .join(format!("{short}-{name}")))
}
