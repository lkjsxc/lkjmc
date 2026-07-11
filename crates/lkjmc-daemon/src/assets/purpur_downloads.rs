use std::path::{Path, PathBuf};

use serde_json::Value;
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
    let target = target_path(&state.jar_root(), &build.md5, &build.name)?;
    let target_text = target.to_string_lossy().to_string();
    let hashes = crate::assets::server_download::download(
        client,
        &target,
        crate::assets::server_download::Request {
            project: "purpur",
            channel,
            url: &build.url,
            expected_size: None,
            sha256: None,
        },
        crate::assets::download_io::ExpectedChecksum::Md5(&build.md5),
    )?;
    if let Some(asset) =
        lkjmc_store::jar::get_by_path(client, &target_text).map_err(|error| error.to_string())?
    {
        return Ok(asset);
    }
    insert_asset(
        client,
        channel,
        &build,
        &target_text,
        &hashes.sha256,
        hashes.size_bytes,
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

fn insert_asset(
    client: &mut postgres::Client,
    channel: &str,
    build: &PurpurBuild,
    path: &str,
    sha256: &str,
    size: i64,
) -> Result<lkjmc_store::jar::JarAssetRecord, String> {
    let id = Uuid::new_v4();
    if let Err(error) = lkjmc_store::jar::insert(
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
    ) {
        return lkjmc_store::jar::get_by_path(client, path)
            .map_err(|lookup| lookup.to_string())?
            .ok_or_else(|| error.to_string());
    }
    lkjmc_store::jar::get(client, id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "inserted Purpur jar asset missing".to_string())
}

fn target_path(root: &str, hash: &str, name: &str) -> Result<PathBuf, String> {
    let short = hash
        .get(0..12)
        .ok_or_else(|| "invalid sha256".to_string())?;
    Ok(Path::new(root)
        .join("purpur")
        .join("purpur")
        .join(format!("{short}-{name}")))
}

struct PurpurBuild {
    name: String,
    url: String,
    md5: String,
}
