use crate::{
    App,
    auth::Actor,
    error::{Error, Result},
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub software: String,
    pub version: String,
    pub java: u32,
    pub url: String,
    pub sha256: String,
}

pub fn load(path: Option<&Path>) -> anyhow::Result<Vec<Preset>> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };
    let values: Vec<Preset> = serde_json::from_slice(&std::fs::read(path)?)?;
    let mut keys = std::collections::BTreeSet::new();
    for value in &values {
        let url = reqwest::Url::parse(&value.url)?;
        anyhow::ensure!(
            url.scheme() == "https" && url.username().is_empty() && url.password().is_none(),
            "Preset requires a public HTTPS artifact"
        );
        anyhow::ensure!(
            value.sha256.len() == 64 && value.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "Preset requires SHA256"
        );
        anyhow::ensure!(
            [8, 17, 21, 25].contains(&value.java) && keys.insert((&value.software, &value.version)),
            "Invalid or duplicated preset"
        );
    }
    Ok(values)
}

pub fn validate(app: &App, software: &str, version: &str, storage_mib: i64) -> Result<()> {
    if !app.config.development && storage_mib < crate::hosting_limits::MIN_SERVER_STORAGE_MIB {
        return Err(Error::invalid(
            "text.servers_need_at_least_16384_mib_of_storage",
        ));
    }
    if software == "custom"
        || app
            .presets
            .iter()
            .any(|p| p.software == software && p.version == version)
    {
        Ok(())
    } else {
        Err(Error::invalid(
            "text.this_server_software_and_version_are_not_available_on_the_host",
        ))
    }
}

pub async fn list(State(app): State<App>, _actor: Actor) -> Json<serde_json::Value> {
    Json(
        serde_json::json!({"minimum_storage_mib": if app.config.development {1024} else {crate::hosting_limits::MIN_SERVER_STORAGE_MIB}, "presets": app.presets.iter().map(|p| serde_json::json!({"software":p.software,"version":p.version,"java":p.java})).collect::<Vec<_>>()}),
    )
}
