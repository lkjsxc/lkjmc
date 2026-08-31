use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{chown, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

use uuid::Uuid;

use crate::error::{OpsError, Result};

pub const MAX_CONTROL_FILE_BYTES: u64 = 1024 * 1024;

pub fn effective_uid() -> u32 {
    rustix::process::geteuid().as_raw()
}

pub fn effective_gid() -> u32 {
    rustix::process::getegid().as_raw()
}

pub fn require_root() -> Result<()> {
    if effective_uid() == 0 {
        Ok(())
    } else {
        Err(OpsError::message("this operation requires root"))
    }
}

pub fn require_absolute_safe(path: &Path, label: &str) -> Result<()> {
    if !path.is_absolute()
        || path == Path::new("/")
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(OpsError::message(format!(
            "{label} must be an absolute normalized non-root path"
        )));
    }
    Ok(())
}

pub fn require_directory(
    path: &Path,
    label: &str,
    expected_uid: Option<u32>,
    expected_gid: Option<u32>,
    exact_mode: Option<u32>,
) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| OpsError::context(&format!("cannot inspect {label}"), error))?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(OpsError::message(format!(
            "{label} is not a non-symlink directory: {}",
            path.display()
        )));
    }
    verify_identity(
        path,
        label,
        &metadata,
        expected_uid,
        expected_gid,
        exact_mode,
    )?;
    Ok(metadata)
}

pub fn require_regular(
    path: &Path,
    label: &str,
    expected_uid: Option<u32>,
    expected_gid: Option<u32>,
    exact_mode: Option<u32>,
    max_bytes: u64,
) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| OpsError::context(&format!("cannot inspect {label}"), error))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(OpsError::message(format!(
            "{label} is not a regular non-symlink file: {}",
            path.display()
        )));
    }
    if metadata.len() > max_bytes {
        return Err(OpsError::message(format!("{label} exceeds its size bound")));
    }
    verify_identity(
        path,
        label,
        &metadata,
        expected_uid,
        expected_gid,
        exact_mode,
    )?;
    Ok(metadata)
}

pub fn read_regular(
    path: &Path,
    label: &str,
    expected_uid: Option<u32>,
    expected_gid: Option<u32>,
    exact_mode: Option<u32>,
    max_bytes: u64,
) -> Result<Vec<u8>> {
    let before = require_regular(
        path,
        label,
        expected_uid,
        expected_gid,
        exact_mode,
        max_bytes,
    )?;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    let mut file = options
        .open(path)
        .map_err(|error| OpsError::context(&format!("cannot open {label}"), error))?;
    let opened = file
        .metadata()
        .map_err(|error| OpsError::context(&format!("cannot re-inspect {label}"), error))?;
    if identity(&before) != identity(&opened) {
        return Err(OpsError::message(format!(
            "{label} identity changed during validation"
        )));
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    file.read_to_end(&mut bytes)
        .map_err(|error| OpsError::context(&format!("cannot read {label}"), error))?;
    if bytes.len() as u64 != opened.len() {
        return Err(OpsError::message(format!(
            "{label} size changed during read"
        )));
    }
    Ok(bytes)
}

pub fn atomic_write(path: &Path, bytes: &[u8], mode: u32, uid: u32, gid: u32) -> Result<()> {
    require_absolute_safe(path, "publication path")?;
    let parent = path
        .parent()
        .ok_or_else(|| OpsError::message("publication path has no parent"))?;
    require_directory(parent, "publication parent", None, None, None)?;
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(OpsError::message(format!(
                "refusing ambiguous publication target: {}",
                path.display()
            )));
        }
    }
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| OpsError::message("publication filename is not UTF-8"))?;
    let temporary = parent.join(format!(".{name}.{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let mut options = OpenOptions::new();
        options
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
        let mut file = options
            .open(&temporary)
            .map_err(|error| OpsError::context("cannot create private publication", error))?;
        file.write_all(bytes)
            .map_err(|error| OpsError::context("cannot write publication", error))?;
        file.sync_all()
            .map_err(|error| OpsError::context("cannot fsync publication", error))?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(mode))
            .map_err(|error| OpsError::context("cannot set publication mode", error))?;
        chown(&temporary, Some(uid), Some(gid))
            .map_err(|error| OpsError::context("cannot set publication ownership", error))?;
        let metadata = file
            .metadata()
            .map_err(|error| OpsError::context("cannot verify publication", error))?;
        verify_identity(
            &temporary,
            "publication temporary file",
            &metadata,
            Some(uid),
            Some(gid),
            Some(mode),
        )?;
        fs::rename(&temporary, path)
            .map_err(|error| OpsError::context("cannot atomically publish file", error))?;
        sync_directory(parent)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub fn create_directory(path: &Path, mode: u32, uid: u32, gid: u32) -> Result<()> {
    fs::create_dir(path)
        .map_err(|error| OpsError::context("cannot create private directory", error))?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|error| OpsError::context("cannot set private directory mode", error))?;
    chown(path, Some(uid), Some(gid))
        .map_err(|error| OpsError::context("cannot set private directory ownership", error))?;
    require_directory(
        path,
        "created private directory",
        Some(uid),
        Some(gid),
        Some(mode),
    )?;
    Ok(())
}

pub fn copy_regular(
    source: &Path,
    destination: &Path,
    mode: u32,
    uid: u32,
    gid: u32,
    max_bytes: u64,
) -> Result<()> {
    let bytes = read_regular(source, "copy source", None, None, None, max_bytes)?;
    atomic_write(destination, &bytes, mode, uid, gid)
}

pub fn sync_directory(path: &Path) -> Result<()> {
    let directory = File::open(path)
        .map_err(|error| OpsError::context("cannot open directory for fsync", error))?;
    directory
        .sync_all()
        .map_err(|error| OpsError::context("cannot fsync directory", error))
}

pub fn validate_ancestry(path: &Path, trusted_root: &Path, expected_uid: u32) -> Result<()> {
    require_absolute_safe(path, "control path")?;
    if !trusted_root.is_absolute()
        || trusted_root
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(OpsError::message(
            "trusted root must be an absolute normalized path",
        ));
    }
    let relative = path
        .strip_prefix(trusted_root)
        .map_err(|_| OpsError::message("control path escapes its trusted root"))?;
    let mut current = PathBuf::from(trusted_root);
    require_directory(&current, "trusted root", Some(expected_uid), None, None)?;
    for component in relative.components() {
        if let Component::Normal(part) = component {
            current.push(part);
            if current == path {
                break;
            }
            let metadata = require_directory(
                &current,
                "control path ancestor",
                Some(expected_uid),
                None,
                None,
            )?;
            if metadata.mode() & 0o022 != 0 {
                return Err(OpsError::message(format!(
                    "control path ancestor is group/other writable: {}",
                    current.display()
                )));
            }
        }
    }
    Ok(())
}

fn verify_identity(
    path: &Path,
    label: &str,
    metadata: &fs::Metadata,
    expected_uid: Option<u32>,
    expected_gid: Option<u32>,
    exact_mode: Option<u32>,
) -> Result<()> {
    if expected_uid.is_some_and(|value| metadata.uid() != value) {
        return Err(OpsError::message(format!(
            "{label} owner differs: {}",
            path.display()
        )));
    }
    if expected_gid.is_some_and(|value| metadata.gid() != value) {
        return Err(OpsError::message(format!(
            "{label} group differs: {}",
            path.display()
        )));
    }
    let mode = metadata.mode() & 0o7777;
    if exact_mode.is_some_and(|value| mode != value) {
        return Err(OpsError::message(format!(
            "{label} mode differs: {}",
            path.display()
        )));
    }
    Ok(())
}

fn identity(metadata: &fs::Metadata) -> (u64, u64, u32, u32, u32, u64) {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.mode(),
        metadata.uid(),
        metadata.gid(),
        metadata.len(),
    )
}
