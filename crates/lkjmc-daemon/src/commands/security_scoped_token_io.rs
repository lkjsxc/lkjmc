use std::fmt::{self, Display};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub(super) struct SecretWriteError {
    detail: String,
    created_file: bool,
}

impl SecretWriteError {
    fn before_create(detail: String) -> Self {
        Self {
            detail,
            created_file: false,
        }
    }

    fn after_create(detail: String) -> Self {
        Self {
            detail,
            created_file: true,
        }
    }

    pub(super) fn created_file(&self) -> bool {
        self.created_file
    }
}

impl Display for SecretWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.detail.fmt(formatter)
    }
}

pub(super) fn write_secret(path: &str, token: &str) -> Result<(), SecretWriteError> {
    let path = Path::new(path);
    let parent = path.parent().ok_or_else(|| {
        SecretWriteError::before_create("credential output has no parent".to_string())
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| SecretWriteError::before_create(error.to_string()))?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| {
            SecretWriteError::before_create(format!("create credential file: {error}"))
        })?;
    file.write_all(token.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all())
        .map_err(|error| SecretWriteError::after_create(format!("write credential file: {error}")))
}

pub(super) fn remove_secret(path: &str) -> Result<(), String> {
    fs::remove_file(path).map_err(|error| format!("remove credential file: {error}"))
}
