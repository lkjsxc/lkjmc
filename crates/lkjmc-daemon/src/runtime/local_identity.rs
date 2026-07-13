use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::runtime::ProcessIdentity;

const MARKER: &str = ".lkjmc-runtime-identity.json";

pub fn write(work_dir: &Path, identity: &ProcessIdentity) -> Result<(), String> {
    let marker = serde_json::json!({
        "pid": identity.pid,
        "executableDevice": identity.executable_device,
        "executableInode": identity.executable_inode,
        "startTicks": identity.start_ticks,
    });
    let bytes = serde_json::to_vec(&marker).map_err(|error| error.to_string())?;
    let temporary = work_dir.join(format!("{MARKER}.{}.tmp", std::process::id()));
    let path = work_dir.join(MARKER);
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|error| format!("create runtime identity marker: {error}"))?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| format!("write runtime identity marker: {error}"))?;
        fs::rename(&temporary, &path)
            .map_err(|error| format!("install runtime identity marker: {error}"))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

pub fn read(root: &Path, id: &str) -> Result<Option<ProcessIdentity>, String> {
    let path = marker_path(root, id);
    let bytes = match fs::read(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("read runtime identity marker: {error}")),
    };
    let marker: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse runtime identity marker: {error}"))?;
    let object = marker
        .as_object()
        .filter(|value| value.len() == 4)
        .ok_or("invalid runtime identity marker")?;
    let pid = object
        .get("pid")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or("invalid runtime identity marker pid")?;
    let field = |name| {
        object
            .get(name)
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| format!("invalid runtime identity marker {name}"))
    };
    Ok(Some(ProcessIdentity {
        pid,
        executable_device: field("executableDevice")?,
        executable_inode: field("executableInode")?,
        start_ticks: field("startTicks")?,
    }))
}

pub fn remove_from(work_dir: &Path) -> Result<(), String> {
    match fs::remove_file(work_dir.join(MARKER)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove runtime identity marker: {error}")),
    }
}

pub fn ids(root: &Path) -> Result<Vec<String>, String> {
    let entries = match fs::read_dir(root) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("list runtime identity markers: {error}")),
    };
    Ok(entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join(MARKER).is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect())
}

fn marker_path(root: &Path, id: &str) -> PathBuf {
    root.join(id).join(MARKER)
}
