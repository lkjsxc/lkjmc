use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub(super) fn write_secret(path: &str, token: &str) -> Result<(), String> {
    let path = Path::new(path);
    let parent = path
        .parent()
        .ok_or_else(|| "token file has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("create token dir: {error}"))?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("token");
    let tmp = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&tmp)
        .map_err(|error| format!("create token tmp: {error}"))?;
    file.write_all(token.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("write token tmp: {error}"))?;
    fs::rename(&tmp, path).map_err(|error| format!("replace token file: {error}"))
}
