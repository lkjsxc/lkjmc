use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub(super) fn write_secret(path: &str, token: &str) -> Result<(), String> {
    let path = Path::new(path);
    let parent = path
        .parent()
        .ok_or_else(|| "credential output has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| format!("create credential file: {error}"))?;
    file.write_all(token.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("write credential file: {error}"))
}

pub(super) fn remove_secret(path: &str) {
    let _ = fs::remove_file(path);
}
