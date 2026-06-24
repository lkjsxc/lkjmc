use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;
use crate::downloads::{BuildInfo, USER_AGENT};

pub fn download_asset(
    state: &AppState,
    client: &mut postgres::Client,
    project: &str,
    channel: &str,
    build: &BuildInfo,
) -> Result<lkjmc_store::jar::JarAssetRecord, String> {
    let target = target_path(&state.jar_root, project, &build.sha256, &build.name)?;
    let target_text = target.to_string_lossy().to_string();
    if let Some(asset) =
        lkjmc_store::jar::get_by_path(client, &target_text).map_err(|error| error.to_string())?
    {
        return Ok(asset);
    }
    fs::create_dir_all(parent(&target)?).map_err(|error| format!("create jar dir: {error}"))?;
    let actual = download_to(&build.url, &target, build.size_bytes)?;
    if !build.sha256.eq_ignore_ascii_case(&actual) {
        let _ = fs::remove_file(&target);
        return Err("download checksum mismatch".to_string());
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
    lkjmc_store::jar::insert(
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
    )
    .map_err(|error| error.to_string())?;
    record_download(client, id, build)?;
    lkjmc_store::jar::get(client, id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "inserted jar asset missing".to_string())
}

fn download_to(url: &str, target: &Path, expected_size: i64) -> Result<String, String> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|error| format!("download jar: {error}"))?;
    let mut reader = response.into_reader();
    let mut file = fs::File::create(target).map_err(|error| format!("create jar: {error}"))?;
    let mut hasher = Sha256::new();
    let mut size = 0_i64;
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        size += i64::try_from(count).map_err(|error| error.to_string())?;
        hasher.update(&buffer[..count]);
        file.write_all(&buffer[..count])
            .map_err(|error| error.to_string())?;
    }
    if size != expected_size {
        return Err(format!("download size mismatch: {size} != {expected_size}"));
    }
    Ok(format!("{:x}", hasher.finalize()))
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

fn parent(path: &Path) -> Result<&Path, String> {
    path.parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))
}
