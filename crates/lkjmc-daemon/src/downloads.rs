use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};

use crate::api;
use crate::app::AppState;
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
    let Some(database_url) = &state.database_url else {
        return Err("Database URL is not configured".to_string());
    };
    let project = body_string(&request.body, "project")?;
    validate_project(&project)?;
    let channel = request
        .body
        .get("channel")
        .and_then(Value::as_str)
        .unwrap_or("stable")
        .to_ascii_uppercase();
    let version = request.body.get("version").and_then(Value::as_str);
    let build = select_build(&project, version, &channel)?;
    let mut client = lkjmc_store::pool::connect(database_url).map_err(|error| error.to_string())?;
    let asset =
        crate::downloads_io::download_asset(state, &mut client, &project, &channel, &build)?;
    Ok(json!({
        "id": asset.id.to_string(),
        "project": asset.project,
        "version": build.version,
        "build": build.build,
        "path": asset.path,
        "sha256": asset.sha256
    }))
}

fn select_build(project: &str, version: Option<&str>, channel: &str) -> Result<BuildInfo, String> {
    let versions = match version {
        Some(version) => vec![version.to_string()],
        None => latest_versions(project)?,
    };
    for value in versions {
        if let Some(build) = latest_stable_build(project, &value, channel)? {
            return Ok(build);
        }
    }
    Err(format!("no {channel} build found for {project}"))
}

fn latest_versions(project: &str) -> Result<Vec<String>, String> {
    let body = get_json(&format!("{BASE}/projects/{project}"))?;
    let versions = body
        .get("versions")
        .and_then(Value::as_object)
        .ok_or_else(|| "PaperMC response missing versions".to_string())?;
    let mut values = versions
        .values()
        .filter_map(Value::as_array)
        .flat_map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
        })
        .collect::<Vec<String>>();
    values.sort_by(|left, right| right.cmp(left));
    Ok(values)
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
        "paper" | "folia" | "velocity" => Ok(()),
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
