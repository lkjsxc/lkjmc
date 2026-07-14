use lkjmc_core::config::LkjmcConfig;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::Fixture;

pub(super) fn repair_proxy(fixture: &mut Fixture) -> Result<(), String> {
    let proxy = asset_path(&fixture.config, "velocity-server")?;
    let valid = fixture.root.join("probe-jar-ProxyProbe/network-probe.jar");
    let bytes = std::fs::read(valid).map_err(|error| error.to_string())?;
    std::fs::write(&proxy, &bytes).map_err(|error| error.to_string())?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    fixture
        .database
        .client_mut()
        .execute(
            "update jar_assets set sha256=$1, size_bytes=$2 where path=$3",
            &[
                &digest,
                &i64::try_from(bytes.len()).map_err(|_| "asset too large")?,
                &proxy,
            ],
        )
        .map_err(|error| error.to_string())?;
    let path = fixture.root.join("lkjmc.json");
    let mut value: Value =
        serde_json::from_slice(&std::fs::read(&path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let assets = value["network"]["assets"]
        .as_array_mut()
        .ok_or("assets missing")?;
    let asset = assets
        .iter_mut()
        .find(|asset| asset["id"] == "velocity-server")
        .ok_or("proxy asset missing")?;
    asset["sha256"] = json!(digest);
    let text = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
    fixture.config = LkjmcConfig::from_json_str(&text).map_err(|error| error.to_string())?;
    std::fs::write(path, text).map_err(|error| error.to_string())
}

fn asset_path(config: &LkjmcConfig, id: &str) -> Result<String, String> {
    config
        .network
        .assets
        .iter()
        .find(|asset| asset.id == id)
        .map(|asset| asset.path.clone())
        .ok_or_else(|| format!("asset missing: {id}"))
}
