use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

use fs2::FileExt;

pub(super) struct ApplyGuard {
    _file: File,
    deadline: Instant,
}

impl ApplyGuard {
    pub fn remaining(&self) -> Result<Duration, String> {
        self.deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| "network apply deadline exceeded".to_string())
    }
}

pub(super) fn acquire(root: &str) -> Result<ApplyGuard, String> {
    acquire_with_deadline(
        root,
        Duration::from_secs(5),
        crate::command_lifecycle::NETWORK_APPLY_DEADLINE,
    )
}

pub(super) fn acquire_with_deadline(
    root: &str,
    lock_wait: Duration,
    total: Duration,
) -> Result<ApplyGuard, String> {
    let path = Path::new(root).join(".network-apply.lock");
    std::fs::create_dir_all(root).map_err(|error| format!("create network root: {error}"))?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&path)
        .map_err(|error| format!("open network apply lock: {error}"))?;
    let wait_deadline = Instant::now() + lock_wait;
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => {
                return Ok(ApplyGuard {
                    _file: file,
                    deadline: Instant::now() + total,
                })
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    && Instant::now() < wait_deadline =>
            {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return Err("network apply lock deadline exceeded".to_string());
            }
            Err(error) => return Err(format!("lock network apply: {error}")),
        }
    }
}
