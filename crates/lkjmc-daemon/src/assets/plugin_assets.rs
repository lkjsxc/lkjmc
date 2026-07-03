use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use lkjmc_core::bootstrap::PluginId;
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;

pub fn register_local(
    state: &AppState,
    client: &mut postgres::Client,
    plugin: PluginId,
) -> Result<lkjmc_store::asset::AssetRecord, String> {
    let spec = local_spec(plugin)?;
    let source = spec.source_path;
    if !source.exists() {
        return Err(format!("built plugin jar missing: {}", source.display()));
    }
    let sha256 = sha256_file(&source)?;
    let target = target_path(&state.asset_root(), spec.platform, spec.project, &sha256)?;
    let target_text = target.to_string_lossy().to_string();
    if let Some(asset) =
        lkjmc_store::asset::get_by_path(client, &target_text).map_err(|error| error.to_string())?
    {
        return Ok(asset);
    }
    fs::create_dir_all(parent(&target)?).map_err(|error| format!("create asset dir: {error}"))?;
    fs::copy(&source, &target).map_err(|error| format!("copy plugin asset: {error}"))?;
    let copied = sha256_file(&target)?;
    if copied != sha256 {
        let _ = fs::remove_file(&target);
        return Err("copied plugin checksum mismatch".to_string());
    }
    let id = Uuid::new_v4();
    lkjmc_store::asset::insert(
        client,
        lkjmc_store::asset::NewAsset {
            id,
            asset_kind: "plugin",
            platform: spec.platform,
            project: spec.project,
            channel: "dev",
            name: spec.project,
            file_name: spec.file_name,
            path: &target_text,
            sha256: &sha256,
            size_bytes: size(&target)?,
            source: "gradle-shadowJar",
            metadata: json!({}),
        },
    )
    .map_err(|error| error.to_string())?;
    lkjmc_store::asset::get(client, id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "inserted plugin asset missing".to_string())
}

struct LocalSpec {
    project: &'static str,
    platform: &'static str,
    file_name: &'static str,
    source_path: PathBuf,
}

fn local_spec(plugin: PluginId) -> Result<LocalSpec, String> {
    match plugin {
        PluginId::LkjmcPaper => Ok(LocalSpec {
            project: "lkjmc-paper",
            platform: "paper",
            file_name: "lkjmc-paper.jar",
            source_path: PathBuf::from("platforms/jvm/paper/build/libs/paper-0.0.0-all.jar"),
        }),
        PluginId::LkjmcVelocity => Ok(LocalSpec {
            project: "lkjmc-velocity",
            platform: "velocity",
            file_name: "lkjmc-velocity.jar",
            source_path: PathBuf::from("platforms/jvm/velocity/build/libs/velocity-0.0.0-all.jar"),
        }),
        other => Err(format!("not a local lkjmc plugin: {}", other.as_str())),
    }
}

fn target_path(root: &str, platform: &str, project: &str, sha256: &str) -> Result<PathBuf, String> {
    let short = sha256
        .get(0..12)
        .ok_or_else(|| "sha256 too short".to_string())?;
    Ok(Path::new(root)
        .join("plugin")
        .join("lkjmc")
        .join(platform)
        .join(format!("{short}-{project}.jar")))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn size(path: &Path) -> Result<i64, String> {
    i64::try_from(fs::metadata(path).map_err(|error| error.to_string())?.len())
        .map_err(|_| "asset is too large".to_string())
}

fn parent(path: &Path) -> Result<&Path, String> {
    path.parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))
}
