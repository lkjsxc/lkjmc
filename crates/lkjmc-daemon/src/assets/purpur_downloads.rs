use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use md5::Md5;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;
use crate::commands::downloads::USER_AGENT;

const BASE: &str = "https://api.purpurmc.org/v2/purpur";
const DEFAULT_JAVA21_RELEASE: &str = "1.21.10";

pub fn sync(
    state: &AppState,
    client: &mut postgres::Client,
    version: Option<&str>,
    channel: &str,
) -> Result<lkjmc_store::jar::JarAssetRecord, String> {
    let build = select_build(version)?;
    let tmp = tmp_path(&state.jar_root(), &build.name)?;
    fs::create_dir_all(parent(&tmp)?).map_err(|error| format!("create purpur dir: {error}"))?;
    let (sha256, size) = download_to(&build.url, &tmp, &build.md5)?;
    let target = target_path(&state.jar_root(), &sha256, &build.name)?;
    if target.exists() {
        let _ = fs::remove_file(&tmp);
        return lkjmc_store::jar::get_by_path(client, &target.to_string_lossy())
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "existing Purpur asset row missing".to_string());
    }
    fs::rename(&tmp, &target).map_err(|error| format!("rename purpur jar: {error}"))?;
    insert_asset(
        client,
        channel,
        &build,
        &target.to_string_lossy(),
        &sha256,
        size,
    )
}

fn select_build(version: Option<&str>) -> Result<PurpurBuild, String> {
    let versions = match version {
        Some(value) => vec![value.to_string()],
        None => {
            let mut values = vec![DEFAULT_JAVA21_RELEASE.to_string()];
            values.extend(latest_versions()?);
            values
        }
    };
    for version in versions {
        if let Ok(build) = latest_build(&version) {
            return Ok(build);
        }
    }
    Err("no Purpur build found".to_string())
}

fn latest_versions() -> Result<Vec<String>, String> {
    let body = get_json(BASE)?;
    let mut values = body
        .get("versions")
        .and_then(Value::as_array)
        .ok_or_else(|| "Purpur response missing versions".to_string())?
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    values.sort_by(|left, right| right.cmp(left));
    Ok(values)
}

fn latest_build(version: &str) -> Result<PurpurBuild, String> {
    let body = get_json(&format!("{BASE}/{version}"))?;
    let build = body
        .get("builds")
        .and_then(|value| value.get("latest"))
        .and_then(Value::as_str)
        .ok_or_else(|| "Purpur response missing latest build".to_string())?;
    let detail = get_json(&format!("{BASE}/{version}/{build}"))?;
    let md5 = detail
        .get("md5")
        .and_then(Value::as_str)
        .ok_or_else(|| "Purpur response missing md5".to_string())?;
    Ok(PurpurBuild {
        name: format!("purpur-{version}-{build}.jar"),
        url: format!("{BASE}/{version}/{build}/download"),
        md5: md5.to_string(),
    })
}

fn get_json(url: &str) -> Result<Value, String> {
    ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|error| format!("Purpur request failed: {error}"))?
        .into_json::<Value>()
        .map_err(|error| format!("Purpur JSON failed: {error}"))
}

fn download_to(url: &str, target: &Path, expected_md5: &str) -> Result<(String, i64), String> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|error| format!("download Purpur jar: {error}"))?;
    let mut reader = response.into_reader();
    let mut file =
        fs::File::create(target).map_err(|error| format!("create Purpur jar: {error}"))?;
    let mut md5 = Md5::new();
    let mut sha = Sha256::new();
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
        md5.update(&buffer[..count]);
        sha.update(&buffer[..count]);
        file.write_all(&buffer[..count])
            .map_err(|error| error.to_string())?;
    }
    let actual_md5 = format!("{:x}", md5.finalize());
    if !actual_md5.eq_ignore_ascii_case(expected_md5) {
        let _ = fs::remove_file(target);
        return Err("Purpur checksum mismatch".to_string());
    }
    Ok((format!("{:x}", sha.finalize()), size))
}

fn insert_asset(
    client: &mut postgres::Client,
    channel: &str,
    build: &PurpurBuild,
    path: &str,
    sha256: &str,
    size: i64,
) -> Result<lkjmc_store::jar::JarAssetRecord, String> {
    let id = Uuid::new_v4();
    lkjmc_store::jar::insert(
        client,
        lkjmc_store::jar::NewJarAsset {
            id,
            kind: "purpur",
            project: "purpur",
            channel,
            name: &build.name,
            path,
            sha256,
            size_bytes: size,
            source: "purpur",
        },
    )
    .map_err(|error| error.to_string())?;
    lkjmc_store::jar::get(client, id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "inserted Purpur jar asset missing".to_string())
}

fn target_path(root: &str, sha256: &str, name: &str) -> Result<PathBuf, String> {
    let short = sha256
        .get(0..12)
        .ok_or_else(|| "invalid sha256".to_string())?;
    Ok(Path::new(root)
        .join("purpur")
        .join("purpur")
        .join(format!("{short}-{name}")))
}

fn tmp_path(root: &str, name: &str) -> Result<PathBuf, String> {
    Ok(Path::new(root)
        .join("purpur")
        .join("tmp")
        .join(format!("{}-{name}", Uuid::new_v4())))
}

fn parent(path: &Path) -> Result<&Path, String> {
    path.parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))
}

struct PurpurBuild {
    name: String,
    url: String,
    md5: String,
}
