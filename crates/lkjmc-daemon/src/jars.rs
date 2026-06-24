use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::body_string;

pub fn handle(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match request.command.as_str() {
        "jar.list" => with_client(state, request, list),
        "jar.import" => with_client(state, request, import),
        "jar.inspect" => with_client(state, request, inspect),
        _ => api::error(request, "command.unknown", "unknown jar command", false),
    }
}

pub fn verified_launch(
    client: &mut postgres::Client,
    asset_id: Uuid,
    memory_mb: i64,
) -> Result<(String, Vec<String>), String> {
    let asset = store(lkjmc_store::jar::get(client, asset_id))?
        .ok_or_else(|| format!("jar asset not found: {asset_id}"))?;
    let actual = sha256_file(Path::new(&asset.path))?;
    if !asset.sha256.eq_ignore_ascii_case(&actual) {
        return Err(format!("jar checksum mismatch: {}", asset.path));
    }
    Ok((
        "java".to_string(),
        vec![
            format!("-Xmx{}M", memory_mb.max(128)),
            "-jar".to_string(),
            asset.path,
            "nogui".to_string(),
        ],
    ))
}

fn with_client<F>(state: &AppState, request: CommandEnvelope, action: F) -> CommandResponse
where
    F: FnOnce(&AppState, CommandEnvelope, &mut postgres::Client) -> Result<CommandResponse, String>,
{
    let Some(database_url) = &state.database_url else {
        return api::error(
            request,
            "database.not_configured",
            "Database URL is not configured",
            false,
        );
    };
    let mut client = match lkjmc_store::pool::connect(database_url) {
        Ok(client) => client,
        Err(error) => return api::error(request, "database.error", error.to_string(), false),
    };
    match action(state, request.clone(), &mut client) {
        Ok(response) => response,
        Err(error) => api::error(request, "jar.error", error, false),
    }
}

fn list(
    _state: &AppState,
    request: CommandEnvelope,
    client: &mut postgres::Client,
) -> Result<CommandResponse, String> {
    let assets = store(lkjmc_store::jar::list(client))?
        .into_iter()
        .map(asset_json)
        .collect::<Vec<Value>>();
    Ok(api::ok(request, json!({"assets": assets})))
}

fn import(
    state: &AppState,
    request: CommandEnvelope,
    client: &mut postgres::Client,
) -> Result<CommandResponse, String> {
    let kind = body_string(&request.body, "kind")?;
    let name = body_string(&request.body, "name")?;
    let source = PathBuf::from(body_string(&request.body, "path")?);
    let canonical = source
        .canonicalize()
        .map_err(|error| format!("canonicalize jar path: {error}"))?;
    let sha256 = sha256_file(&canonical)?;
    let size = i64::try_from(
        fs::metadata(&canonical)
            .map_err(|error| error.to_string())?
            .len(),
    )
    .map_err(|_| "jar is too large".to_string())?;
    let target = target_path(&state.jar_root, &kind, &name, &sha256)?;
    if target.exists() {
        return Err(format!("jar target already exists: {}", target.display()));
    }
    fs::create_dir_all(parent(&target)?).map_err(|error| format!("create jar dir: {error}"))?;
    fs::copy(&canonical, &target).map_err(|error| format!("copy jar: {error}"))?;
    let id = Uuid::new_v4();
    let target_text = target.to_string_lossy().to_string();
    if let Err(error) = lkjmc_store::jar::insert(
        client,
        lkjmc_store::jar::NewJarAsset {
            id,
            kind: &kind,
            project: &kind,
            channel: "imported",
            name: &name,
            path: &target_text,
            sha256: &sha256,
            size_bytes: size,
            source: "manual",
        },
    ) {
        let _ = fs::remove_file(&target);
        return Err(error.to_string());
    }
    Ok(api::ok(
        request,
        json!({"id": id.to_string(), "path": target_text, "sha256": sha256}),
    ))
}

fn inspect(
    _state: &AppState,
    request: CommandEnvelope,
    client: &mut postgres::Client,
) -> Result<CommandResponse, String> {
    let query = body_string(&request.body, "query")?;
    let asset = store(lkjmc_store::jar::latest_matching(client, &query))?
        .ok_or_else(|| format!("jar asset not found: {query}"))?;
    Ok(api::ok(request, asset_json(asset)))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| format!("open jar: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("read jar: {error}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn target_path(root: &str, kind: &str, name: &str, sha256: &str) -> Result<PathBuf, String> {
    let short = sha256
        .get(0..12)
        .ok_or_else(|| "invalid sha256".to_string())?;
    Ok(Path::new(root)
        .join("custom")
        .join(safe_name(kind))
        .join(format!("{}-{}", short, safe_name(name))))
}

fn safe_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '.' | '_' | '-' => ch,
            _ => '_',
        })
        .collect()
}

fn parent(path: &Path) -> Result<&Path, String> {
    path.parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))
}

fn asset_json(asset: lkjmc_store::jar::JarAssetRecord) -> Value {
    json!({
        "id": asset.id.to_string(),
        "kind": asset.kind,
        "project": asset.project,
        "channel": asset.channel,
        "name": asset.name,
        "path": asset.path,
        "sha256": asset.sha256,
        "sizeBytes": asset.size_bytes,
        "source": asset.source
    })
}

fn store<T>(result: Result<T, lkjmc_store::error::StoreError>) -> Result<T, String> {
    result.map_err(|error| error.to_string())
}
