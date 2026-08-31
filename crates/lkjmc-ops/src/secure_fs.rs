use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{chown, MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

use rustix::fs::{AtFlags, FileType, Mode, OFlags, ResolveFlags};
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
    let (parent, name) = open_parent_nofollow(path, label)?;
    let opened = rustix::fs::openat(
        &parent,
        name.as_str(),
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|error| OpsError::context(&format!("cannot open {label}"), error))?;
    let mut file = File::from(opened);
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
    let after = file
        .metadata()
        .map_err(|error| OpsError::context(&format!("cannot finally inspect {label}"), error))?;
    if bytes.len() as u64 != opened.len() || identity(&opened) != identity(&after) {
        return Err(OpsError::message(format!(
            "{label} identity or size changed during read"
        )));
    }
    Ok(bytes)
}

pub fn atomic_write(path: &Path, bytes: &[u8], mode: u32, uid: u32, gid: u32) -> Result<()> {
    require_absolute_safe(path, "publication path")?;
    let parent = path
        .parent()
        .ok_or_else(|| OpsError::message("publication path has no parent"))?;
    let (parent_fd, name) = open_parent_nofollow(path, "publication path")?;
    match rustix::fs::statat(&parent_fd, name.as_str(), AtFlags::SYMLINK_NOFOLLOW) {
        Ok(metadata) => {
            if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile {
                return Err(OpsError::message(format!(
                    "refusing ambiguous publication target: {}",
                    path.display()
                )));
            }
        }
        Err(rustix::io::Errno::NOENT) => {}
        Err(error) => {
            return Err(OpsError::context(
                "cannot inspect publication target",
                error,
            ));
        }
    }
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(OpsError::message(format!(
                "refusing ambiguous publication target: {}",
                path.display()
            )));
        }
    }
    let temporary = format!(".{name}.{}.tmp", Uuid::new_v4());
    let result = (|| {
        let temporary_fd = rustix::fs::openat(
            &parent_fd,
            temporary.as_str(),
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|error| OpsError::context("cannot create private publication", error))?;
        let mut file = File::from(temporary_fd);
        file.write_all(bytes)
            .map_err(|error| OpsError::context("cannot write publication", error))?;
        file.sync_all()
            .map_err(|error| OpsError::context("cannot fsync publication", error))?;
        rustix::fs::fchown(
            &file,
            Some(rustix::process::Uid::from_raw(uid)),
            Some(rustix::process::Gid::from_raw(gid)),
        )
        .map_err(|error| OpsError::context("cannot set publication ownership", error))?;
        rustix::fs::fchmod(&file, Mode::from_raw_mode(mode))
            .map_err(|error| OpsError::context("cannot set publication mode", error))?;
        file.sync_all()
            .map_err(|error| OpsError::context("cannot fsync publication metadata", error))?;
        let metadata = file
            .metadata()
            .map_err(|error| OpsError::context("cannot verify publication", error))?;
        verify_identity(
            path,
            "publication temporary file",
            &metadata,
            Some(uid),
            Some(gid),
            Some(mode),
        )?;
        rustix::fs::renameat(&parent_fd, temporary.as_str(), &parent_fd, name.as_str())
            .map_err(|error| OpsError::context("cannot atomically publish file", error))?;
        rustix::fs::fsync(&parent_fd)
            .map_err(|error| OpsError::context("cannot fsync publication directory", error))?;
        let current_parent = require_directory(parent, "publication parent", None, None, None)?;
        let opened_parent = rustix::fs::fstat(&parent_fd)
            .map_err(|error| OpsError::context("cannot reinspect publication parent", error))?;
        if current_parent.dev() != opened_parent.st_dev as u64
            || current_parent.ino() != opened_parent.st_ino as u64
        {
            return Err(OpsError::message(
                "publication parent identity changed during publication",
            ));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = rustix::fs::unlinkat(&parent_fd, temporary.as_str(), AtFlags::empty());
    }
    result
}

pub fn atomic_symlink(target: &Path, destination: &Path, uid: u32) -> Result<()> {
    require_absolute_safe(destination, "symlink publication path")?;
    let mut target_components = target.components();
    if target.is_absolute()
        || !matches!(target_components.next(), Some(Component::Normal(_)))
        || target_components.next().is_some()
    {
        return Err(OpsError::message(
            "symlink publication target must be one relative path component",
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| OpsError::message("symlink publication path has no parent"))?;
    let parent_metadata =
        require_directory(parent, "symlink publication parent", Some(uid), None, None)?;
    if parent_metadata.mode() & 0o022 != 0 {
        return Err(OpsError::message(
            "symlink publication parent is group/other writable",
        ));
    }
    let (parent_fd, name) = open_parent_nofollow(destination, "symlink publication path")?;
    match rustix::fs::statat(&parent_fd, name.as_str(), AtFlags::SYMLINK_NOFOLLOW) {
        Ok(metadata) => {
            if FileType::from_raw_mode(metadata.st_mode) != FileType::Symlink
                || metadata.st_uid != uid
            {
                return Err(OpsError::message(
                    "existing symlink publication target is not an owned symlink",
                ));
            }
        }
        Err(rustix::io::Errno::NOENT) => {}
        Err(error) => {
            return Err(OpsError::context(
                "cannot inspect symlink publication target",
                error,
            ));
        }
    }
    let temporary = format!(".{name}.{}.tmp", Uuid::new_v4());
    let operation = (|| {
        rustix::fs::symlinkat(target, &parent_fd, temporary.as_str())
            .map_err(|error| OpsError::context("cannot create release pointer", error))?;
        let metadata =
            rustix::fs::statat(&parent_fd, temporary.as_str(), AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|error| OpsError::context("cannot inspect release pointer", error))?;
        if FileType::from_raw_mode(metadata.st_mode) != FileType::Symlink || metadata.st_uid != uid
        {
            return Err(OpsError::message(
                "new release pointer is not an owned symlink",
            ));
        }
        rustix::fs::renameat(&parent_fd, temporary.as_str(), &parent_fd, name.as_str())
            .map_err(|error| OpsError::context("cannot publish release pointer", error))?;
        rustix::fs::fsync(&parent_fd)
            .map_err(|error| OpsError::context("cannot fsync release pointer directory", error))?;
        let published = rustix::fs::readlinkat(&parent_fd, name.as_str(), Vec::new())
            .map_err(|error| OpsError::context("cannot verify release pointer", error))?;
        if published.to_bytes() != target.as_os_str().as_bytes() {
            return Err(OpsError::message(
                "published release pointer target differs from the requested target",
            ));
        }
        let current_parent =
            require_directory(parent, "symlink publication parent", Some(uid), None, None)?;
        let opened_parent = rustix::fs::fstat(&parent_fd)
            .map_err(|error| OpsError::context("cannot reinspect symlink parent", error))?;
        if current_parent.dev() != opened_parent.st_dev as u64
            || current_parent.ino() != opened_parent.st_ino as u64
        {
            return Err(OpsError::message(
                "symlink publication parent identity changed during publication",
            ));
        }
        Ok(())
    })();
    if operation.is_err() {
        let _ = rustix::fs::unlinkat(&parent_fd, temporary.as_str(), AtFlags::empty());
    }
    operation
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

fn open_parent_nofollow(path: &Path, label: &str) -> Result<(rustix::fd::OwnedFd, String)> {
    require_absolute_safe(path, label)?;
    let parent = path
        .parent()
        .ok_or_else(|| OpsError::message(format!("{label} has no parent")))?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty() && !value.contains('/'))
        .ok_or_else(|| OpsError::message(format!("{label} filename is not safe UTF-8")))?
        .to_string();
    let root = rustix::fs::open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| OpsError::context("cannot open filesystem root", error))?;
    let relative = parent
        .strip_prefix(Path::new("/"))
        .map_err(|_| OpsError::message(format!("{label} parent escapes filesystem root")))?;
    let relative = if relative.as_os_str().is_empty() {
        Path::new(".")
    } else {
        relative
    };
    let directory = rustix::fs::openat2(
        &root,
        relative,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_MAGICLINKS | ResolveFlags::NO_SYMLINKS,
    )
    .map_err(|error| {
        OpsError::context(&format!("cannot open {label} parent without links"), error)
    })?;
    let path_metadata = require_directory(parent, &format!("{label} parent"), None, None, None)?;
    let opened = rustix::fs::fstat(&directory)
        .map_err(|error| OpsError::context(&format!("cannot inspect {label} parent"), error))?;
    if path_metadata.dev() != opened.st_dev as u64 || path_metadata.ino() != opened.st_ino as u64 {
        return Err(OpsError::message(format!(
            "{label} parent identity changed during validation"
        )));
    }
    Ok((directory, name))
}

fn identity(metadata: &fs::Metadata) -> (u64, u64, u32, u32, u32, u64, i64, i64, i64, i64) {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.mode(),
        metadata.uid(),
        metadata.gid(),
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    )
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{symlink, PermissionsExt};

    use super::*;

    #[test]
    fn descriptor_relative_publication_rejects_a_symlinked_ancestor() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("lkjmc-secure-fs-{}", uuid::Uuid::new_v4().simple()));
        let managed = root.join("managed");
        let unrelated = root.join("unrelated");
        fs::create_dir_all(&managed)?;
        fs::create_dir(&unrelated)?;
        fs::set_permissions(&managed, fs::Permissions::from_mode(0o700))?;
        fs::set_permissions(&unrelated, fs::Permissions::from_mode(0o700))?;
        symlink(&unrelated, managed.join("instance"))?;
        let target = managed.join("instance/eula.txt");
        let error = atomic_write(
            &target,
            b"eula=true\n",
            0o600,
            effective_uid(),
            effective_gid(),
        )
        .err()
        .ok_or_else(|| OpsError::message("symlinked publication ancestor unexpectedly passed"))?;
        assert!(error.to_string().contains("without links"));
        assert!(!unrelated.join("eula.txt").exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn descriptor_relative_symlink_publication_replaces_only_an_owned_pointer() -> Result<()> {
        let root =
            std::env::temp_dir().join(format!("lkjmc-secure-link-{}", Uuid::new_v4().simple()));
        fs::create_dir(&root)?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
        let pointer = root.join("current");
        atomic_symlink(Path::new("a1b2"), &pointer, effective_uid())?;
        assert_eq!(fs::read_link(&pointer)?, Path::new("a1b2"));
        atomic_symlink(Path::new("c3d4"), &pointer, effective_uid())?;
        assert_eq!(fs::read_link(&pointer)?, Path::new("c3d4"));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
