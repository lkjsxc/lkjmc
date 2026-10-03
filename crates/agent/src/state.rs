use anyhow::{Result, ensure};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub struct Store {
    pub root: PathBuf,
    _lock: File,
}
impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root)?;
        ensure!(
            !std::fs::symlink_metadata(root)?.file_type().is_symlink(),
            "Agent state cannot be a symlink"
        );
        std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o700))?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(root.join("agent.lock"))?;
        let metadata = lock.metadata()?;
        ensure!(
            metadata.is_file() && metadata.nlink() == 1 && metadata.mode() & 0o077 == 0,
            "Unsafe agent state lock"
        );
        lock.try_lock()
            .map_err(|_| anyhow::anyhow!("Another agent owns this state directory"))?;
        for p in [
            "bindings",
            "inspections",
            "jobs",
            "downloads",
            "backups",
            "prunes",
            "operations",
        ] {
            std::fs::create_dir_all(root.join(p))?;
            ensure!(
                std::fs::symlink_metadata(root.join(p))?.is_dir(),
                "Agent state group must be a real directory"
            );
        }
        File::open(root)?.sync_all()?;
        Ok(Self {
            root: root.into(),
            _lock: lock,
        })
    }
    pub fn read<T: DeserializeOwned>(&self, group: &str, id: Uuid) -> Result<Option<T>> {
        let path = self.root.join(group).join(format!("{id}.json"));
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    pub fn write<T: Serialize>(&self, group: &str, id: Uuid, value: &T) -> Result<()> {
        atomic(
            &self.root.join(group).join(format!("{id}.json")),
            &serde_json::to_vec(value)?,
        )
    }
    pub fn all<T: DeserializeOwned>(&self, group: &str) -> Result<Vec<T>> {
        let mut result = Vec::new();
        for entry in std::fs::read_dir(self.root.join(group))? {
            let path = entry?.path();
            if path.extension().is_some_and(|x| x == "json") {
                result.push(serde_json::from_slice(&std::fs::read(path)?)?);
            }
        }
        Ok(result)
    }
}
pub fn atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Missing parent"))?;
    let temporary = parent.join(format!(".tmp-{}", Uuid::new_v4()));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    std::fs::rename(temporary, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}
