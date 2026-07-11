use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

pub fn create_private(path: &Path, content: &[u8]) -> Result<bool, String> {
    match private_open(path) {
        Ok(file) => {
            write_and_sync(file, content, path)?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(format!("create {}: {error}", path.display())),
    }
}

pub fn replace_private(path: &Path, content: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("secret path has no parent: {}", path.display()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("secret path has no file name: {}", path.display()))?;
    let temp = parent.join(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed),
    ));
    let result = private_open(&temp)
        .map_err(|error| format!("create {}: {error}", temp.display()))
        .and_then(|file| write_and_sync(file, content, &temp))
        .and_then(|_| fs::rename(&temp, path).map_err(|error| format!("rename secret: {error}")));
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

fn private_open(path: &Path) -> std::io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

fn write_and_sync(mut file: File, content: &[u8], path: &Path) -> Result<(), String> {
    file.write_all(content)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("write {}: {error}", path.display()))
}
