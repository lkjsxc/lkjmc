use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::os::fd::AsRawFd;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

pub(in crate::support::bundle) struct VerifiedOutput {
    pub(in crate::support::bundle) directory: File,
    descriptor_path: PathBuf,
    name: OsString,
}

impl VerifiedOutput {
    pub(in crate::support::bundle) fn new(output: &Path) -> Result<Self, String> {
        let name = output
            .file_name()
            .filter(|name| !name.is_empty())
            .ok_or("support archive filename is unavailable")?
            .to_os_string();
        let parent = output
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let canonical = canonical_directory(parent)?;
        let expected =
            fs::symlink_metadata(&canonical).map_err(|_| "support archive parent unavailable")?;
        let directory = File::open(&canonical).map_err(|_| "support archive parent unavailable")?;
        let opened = directory
            .metadata()
            .map_err(|_| "inspect support archive parent failed")?;
        if opened.dev() != expected.dev() || opened.ino() != expected.ino() {
            return Err("support archive parent changed while opening".into());
        }
        let descriptor_path = PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()));
        let verified = Self {
            directory,
            descriptor_path,
            name,
        };
        verified.reject_existing_target()?;
        Ok(verified)
    }

    pub(in crate::support::bundle) fn child(&self, name: &OsStr) -> PathBuf {
        self.descriptor_path.join(name)
    }

    pub(in crate::support::bundle) fn target(&self) -> PathBuf {
        self.child(&self.name)
    }

    fn reject_existing_target(&self) -> Result<(), String> {
        match fs::symlink_metadata(self.target()) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                Err("support archive target symlink is forbidden".into())
            }
            Ok(metadata) if !metadata.file_type().is_file() => {
                Err("support archive target is not a regular file".into())
            }
            Ok(_) => Err("support archive destination exists".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("inspect support archive target failed".into()),
        }
    }
}

pub(in crate::support::bundle) fn canonical_directory(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| "support archive current directory unavailable")?
            .join(path)
    };
    let mut lexical = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::RootDir => lexical.push(Path::new("/")),
            Component::Normal(value) => {
                lexical.push(value);
                let metadata = fs::symlink_metadata(&lexical)
                    .map_err(|_| "support archive parent unavailable")?;
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err("support archive parent component is not a real directory".into());
                }
            }
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) => {
                return Err("support archive traversal is forbidden".into())
            }
        }
    }
    let canonical = fs::canonicalize(&lexical).map_err(|_| "support archive parent unavailable")?;
    if canonical != lexical {
        return Err("support archive parent violates lexical root policy".into());
    }
    Ok(canonical)
}
