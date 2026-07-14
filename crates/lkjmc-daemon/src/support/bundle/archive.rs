use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use uuid::Uuid;

pub(super) fn validate_output(output: &Path) -> Result<(), String> {
    if output
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err("support archive traversal is forbidden".into());
    }
    if fs::symlink_metadata(output).is_ok() {
        return Err("support archive destination exists".into());
    }
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let metadata = fs::metadata(parent).map_err(|_| "support archive parent unavailable")?;
    if !metadata.is_dir() {
        return Err("support archive parent is not a directory".into());
    }
    Ok(())
}

pub(super) fn write(
    output: &Path,
    entries: &[(String, Vec<u8>)],
    started: Instant,
    time_cap: Duration,
) -> Result<(), String> {
    let temp = temporary_path(output);
    let result = write_inner(&temp, output, entries, started, time_cap);
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn write_inner(
    temp: &Path,
    output: &Path,
    entries: &[(String, Vec<u8>)],
    started: Instant,
    time_cap: Duration,
) -> Result<(), String> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(temp)
        .map_err(|_| "create private support archive failed")?;
    let mut archive = tar::Builder::new(file);
    for (name, value) in entries {
        check_time(started, time_cap)?;
        if crate::support::redaction::contains_sensitive_canary(value) {
            return Err("support archive secret canary detected".into());
        }
        append(&mut archive, name, value)?;
    }
    archive
        .finish()
        .map_err(|_| "finish support archive failed")?;
    let mut file = archive
        .into_inner()
        .map_err(|_| "close support archive failed")?;
    file.flush().map_err(|_| "flush support archive failed")?;
    file.sync_all().map_err(|_| "sync support archive failed")?;
    drop(file);
    final_scan(temp)?;
    fs::hard_link(temp, output).map_err(|_| "publish support archive failed")?;
    fs::remove_file(temp).map_err(|_| "remove support archive temporary failed")?;
    fs::set_permissions(output, fs::Permissions::from_mode(0o600))
        .map_err(|_| "set support archive permissions failed")?;
    fs::File::open(output.parent().unwrap_or_else(|| Path::new(".")))
        .and_then(|file| file.sync_all())
        .map_err(|_| "sync support archive directory failed")?;
    Ok(())
}

fn append(archive: &mut tar::Builder<fs::File>, name: &str, value: &[u8]) -> Result<(), String> {
    let mut header = tar::Header::new_gnu();
    header.set_size(value.len() as u64);
    header.set_mode(0o600);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_cksum();
    archive
        .append_data(&mut header, name, value)
        .map_err(|_| "write support member failed".to_string())
}

fn final_scan(temp: &Path) -> Result<(), String> {
    let mut retained = Vec::new();
    fs::File::open(temp)
        .and_then(|mut file| file.read_to_end(&mut retained))
        .map_err(|_| "scan support archive failed")?;
    if crate::support::redaction::contains_sensitive_canary(&retained) {
        return Err("support archive final canary scan failed".into());
    }
    Ok(())
}

fn temporary_path(output: &Path) -> PathBuf {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    parent.join(format!(".lkjmc-support-{}.tmp", Uuid::new_v4().simple()))
}

fn check_time(started: Instant, cap: Duration) -> Result<(), String> {
    if started.elapsed() > cap {
        Err("support bundle time cap exceeded".into())
    } else {
        Ok(())
    }
}
