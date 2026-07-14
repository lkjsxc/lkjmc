use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use lkjmc_core::plugin::PluginId;
use sha2::{Digest, Sha256};

use crate::app::AppState;

pub fn install(
    state: &AppState,
    client: &mut postgres::Client,
    instance_id: &str,
    plugin: PluginId,
) -> Result<PathBuf, String> {
    let asset = lkjmc_store::asset::latest_for_project(client, "plugin", project(plugin))
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("plugin asset not found: {}", plugin.as_str()))?;
    let source = Path::new(&asset.path);
    let source_hash = sha256_file(source)?;
    if !asset.sha256.eq_ignore_ascii_case(&source_hash) {
        return Err(format!("plugin asset checksum mismatch: {}", asset.path));
    }
    let target = target_path(&state.data_root(), instance_id, plugin);
    let plugins_dir = target
        .parent()
        .ok_or_else(|| "plugin target has no parent".to_string())?;
    fs::create_dir_all(plugins_dir).map_err(|error| format!("create plugin dir: {error}"))?;
    fs::copy(source, &target).map_err(|error| format!("copy plugin: {error}"))?;
    let target_hash = sha256_file(&target)?;
    if !asset.sha256.eq_ignore_ascii_case(&target_hash) {
        let _ = fs::remove_file(&target);
        return Err("installed plugin checksum mismatch".to_string());
    }
    let target_text = target.to_string_lossy().to_string();
    lkjmc_store::plugin::upsert_installation(
        client,
        lkjmc_store::plugin::UpsertPluginInstallation {
            instance_id,
            plugin_id: plugin.as_str(),
            asset_id: asset.id,
            target_path: &target_text,
            installed_sha256: &target_hash,
        },
    )
    .map_err(|error| error.to_string())?;
    Ok(target)
}

pub fn target_path(data_root: &str, instance_id: &str, plugin: PluginId) -> PathBuf {
    Path::new(data_root)
        .join(instance_id)
        .join("plugins")
        .join(target_name(plugin))
}

pub fn target_name(plugin: PluginId) -> &'static str {
    match plugin {
        PluginId::LkjmcPaper => "lkjmc-paper.jar",
        PluginId::LkjmcVelocity => "lkjmc-velocity.jar",
        PluginId::ViaVersion => "ViaVersion.jar",
        PluginId::ViaBackwards => "ViaBackwards.jar",
        PluginId::Geyser => "Geyser-Velocity.jar",
        PluginId::Floodgate => "floodgate-velocity.jar",
    }
}

fn project(plugin: PluginId) -> &'static str {
    plugin.as_str()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_plugin_target_names() {
        assert_eq!(target_name(PluginId::LkjmcPaper), "lkjmc-paper.jar");
        assert_eq!(target_name(PluginId::LkjmcVelocity), "lkjmc-velocity.jar");
        assert_eq!(target_name(PluginId::ViaVersion), "ViaVersion.jar");
        assert_eq!(target_name(PluginId::ViaBackwards), "ViaBackwards.jar");
        assert_eq!(target_name(PluginId::Geyser), "Geyser-Velocity.jar");
        assert_eq!(target_name(PluginId::Floodgate), "floodgate-velocity.jar");
    }

    #[test]
    fn maps_managed_plugin_directories() {
        assert_eq!(
            target_path("/var/lib/lkjmc/instances", "hub", PluginId::LkjmcPaper),
            Path::new("/var/lib/lkjmc/instances/hub/plugins/lkjmc-paper.jar")
        );
        assert_eq!(
            target_path("/var/lib/lkjmc/instances", "proxy", PluginId::LkjmcVelocity),
            Path::new("/var/lib/lkjmc/instances/proxy/plugins/lkjmc-velocity.jar")
        );
    }
}
