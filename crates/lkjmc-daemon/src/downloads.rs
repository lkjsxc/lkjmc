use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;
use crate::downloads_versions::candidate_versions;
use crate::instance_helpers::body_string;

pub(crate) const USER_AGENT: &str = "lkjmc (+https://github.com/lkjsxc/lkjmc)";
const BASE: &str = "https://fill.papermc.io/v3";

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match sync(state, request.clone()) {
        Ok(body) => api::ok(request, body),
        Err(error) => api::error(request, "jar.sync_failed", error, false),
    }
}

fn sync(state: &AppState, request: CommandEnvelope) -> Result<Value, String> {
    if state.database_url().is_none() {
        return Err("Database URL is not configured".to_string());
    }
    let project = body_string(&request.body, "project")?;
    validate_project(&project)?;
    let channel = request
        .body
        .get("channel")
        .and_then(Value::as_str)
        .unwrap_or("stable")
        .to_ascii_uppercase();
    let minecraft_release = request.body.get("minecraftRelease").and_then(Value::as_str);
    let mut client = state.database_connection()?;
    if project == "purpur" {
        let asset = crate::purpur_downloads::sync(
            state,
            &mut client,
            minecraft_release,
            &channel.to_ascii_lowercase(),
        )?;
        return Ok(json!({
            "id": asset.id.to_string(),
            "project": asset.project,
            "minecraftRelease": minecraft_release,
            "build": null,
            "path": asset.path,
            "sha256": asset.sha256
        }));
    }
    let build = select_build(&project, minecraft_release, &channel)?;
    let asset =
        crate::downloads_io::download_asset(state, &mut client, &project, &channel, &build)?;
    Ok(json!({
        "id": asset.id.to_string(),
        "project": asset.project,
        "minecraftRelease": build.version,
        "build": build.build,
        "path": asset.path,
        "sha256": asset.sha256
    }))
}

fn select_build(project: &str, version: Option<&str>, channel: &str) -> Result<BuildInfo, String> {
    let available = if version.is_some() {
        Vec::new()
    } else {
        latest_versions(project)?
    };
    let versions = candidate_versions(project, version, available);
    for value in versions {
        match latest_stable_build(project, &value, channel) {
            Ok(Some(build)) => return Ok(build),
            Ok(None) => {}
            Err(error) if version.is_none() && missing_version(&error) => {}
            Err(error) => return Err(error),
        }
    }
    Err(format!("no {channel} build found for {project}"))
}

fn latest_versions(project: &str) -> Result<Vec<String>, String> {
    let body = get_json(&format!("{BASE}/projects/{project}"))?;
    let versions = body
        .get("versions")
        .ok_or_else(|| "PaperMC response missing versions".to_string())?;
    let values = match versions {
        Value::Array(items) => version_strings(items),
        Value::Object(groups) => groups
            .values()
            .filter_map(Value::as_array)
            .flat_map(|items| version_strings(items))
            .collect(),
        _ => return Err("PaperMC response missing versions".to_string()),
    };
    Ok(crate::downloads_versions::newest_first(values))
}

fn version_strings(items: &[Value]) -> Vec<String> {
    items
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn missing_version(error: &str) -> bool {
    error.contains("status code 404")
}

fn latest_stable_build(
    project: &str,
    version: &str,
    channel: &str,
) -> Result<Option<BuildInfo>, String> {
    let body = get_json(&format!(
        "{BASE}/projects/{project}/versions/{version}/builds"
    ))?;
    let builds = body
        .as_array()
        .ok_or_else(|| "PaperMC builds response was not an array".to_string())?;
    let mut selected = None;
    for build in builds
        .iter()
        .filter(|build| channel_matches(build, channel))
    {
        let candidate = build_info(project, version, channel, build)?;
        if selected
            .as_ref()
            .is_none_or(|current: &BuildInfo| candidate.build > current.build)
        {
            selected = Some(candidate);
        }
    }
    Ok(selected)
}

fn build_info(
    project: &str,
    version: &str,
    channel: &str,
    build: &Value,
) -> Result<BuildInfo, String> {
    let download = build
        .get("downloads")
        .and_then(|value| value.get("server:default"))
        .ok_or_else(|| "build missing server download".to_string())?;
    Ok(BuildInfo {
        project: project.to_string(),
        version: version.to_string(),
        channel: channel.to_ascii_lowercase(),
        build: build.get("id").and_then(Value::as_i64).unwrap_or_default(),
        name: body_string(download, "name")?,
        url: body_string(download, "url")?,
        sha256: download
            .get("checksums")
            .and_then(|value| value.get("sha256"))
            .and_then(Value::as_str)
            .ok_or_else(|| "download missing sha256".to_string())?
            .to_string(),
        size_bytes: download
            .get("size")
            .and_then(Value::as_i64)
            .ok_or_else(|| "download missing size".to_string())?,
    })
}

fn get_json(url: &str) -> Result<Value, String> {
    ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|error| format!("PaperMC request failed: {error}"))?
        .into_json::<Value>()
        .map_err(|error| format!("PaperMC JSON failed: {error}"))
}

fn channel_matches(build: &Value, channel: &str) -> bool {
    build
        .get("channel")
        .and_then(Value::as_str)
        .is_some_and(|value| value.eq_ignore_ascii_case(channel))
}

fn validate_project(project: &str) -> Result<(), String> {
    match project {
        "paper" | "folia" | "purpur" | "velocity" => Ok(()),
        _ => Err(format!("unsupported PaperMC project: {project}")),
    }
}

pub(crate) struct BuildInfo {
    pub(crate) project: String,
    pub(crate) version: String,
    pub(crate) channel: String,
    pub(crate) build: i64,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: i64,
}
