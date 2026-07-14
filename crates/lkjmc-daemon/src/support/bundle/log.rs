use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use super::{archive, Deadline, BYTE_CAP};
use crate::app::AppState;

pub(super) fn allowlisted(state: &AppState, deadline: Deadline) -> Result<Option<Vec<u8>>, String> {
    let path = Path::new(&state.log_root()).join("daemon.log");
    deadline.check()?;
    let mut file = match open_regular_nofollow(&path)? {
        Some(value) => value,
        None => return Ok(None),
    };
    let mut value = Vec::new();
    let mut chunk = [0_u8; 16 * 1024];
    loop {
        deadline.check()?;
        let count = file
            .read(&mut chunk)
            .map_err(|_| "read allowlisted log failed")?;
        if count == 0 {
            break;
        }
        value.extend_from_slice(&chunk[..count]);
        if value.len() > BYTE_CAP {
            return Err("allowlisted log grew beyond byte cap".into());
        }
    }
    deadline.check()?;
    Ok(Some(crate::support::redaction::text_bytes(&value)))
}

fn open_regular_nofollow(path: &Path) -> Result<Option<File>, String> {
    let parent = path.parent().ok_or("allowlisted log parent unavailable")?;
    let parent = archive::canonical_directory(parent)?;
    let expected = fs::symlink_metadata(&parent).map_err(|_| "inspect log parent failed")?;
    let directory = File::open(parent).map_err(|_| "open log parent failed")?;
    let opened_parent = directory
        .metadata()
        .map_err(|_| "inspect opened log parent failed")?;
    if (expected.dev(), expected.ino()) != (opened_parent.dev(), opened_parent.ino()) {
        return Err("allowlisted log parent changed while opening".into());
    }
    let target = PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()))
        .join(path.file_name().ok_or("allowlisted log name unavailable")?);
    let metadata = match fs::symlink_metadata(&target) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("inspect allowlisted log failed".into()),
    };
    if !metadata.file_type().is_file() || metadata.len() > BYTE_CAP as u64 {
        return Err("allowlisted log is not a bounded regular file".into());
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(target)
        .map_err(|_| "open allowlisted log failed")?;
    let opened = file
        .metadata()
        .map_err(|_| "inspect opened allowlisted log failed")?;
    if !opened.is_file() || opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return Err("allowlisted log changed while opening".into());
    }
    Ok(Some(file))
}
