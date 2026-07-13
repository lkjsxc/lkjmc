use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::runtime::ProcessIdentity;

pub fn terminate_group(pid: u32) -> bool {
    signal_group(pid, "-TERM")
}

pub fn kill_group(pid: u32) -> bool {
    signal_group(pid, "-KILL")
}

pub fn group_exists(pid: u32) -> bool {
    signal_group(pid, "-0")
}

pub fn resolve_executable(command: &str) -> Result<PathBuf, String> {
    let candidate = Path::new(command);
    if candidate.components().count() > 1 {
        return fs::canonicalize(candidate).map_err(|error| format!("resolve executable: {error}"));
    }
    let path = std::env::var_os("PATH").ok_or("PATH is unavailable")?;
    for root in std::env::split_paths(&path) {
        let candidate = root.join(command);
        if candidate.is_file() {
            return fs::canonicalize(candidate)
                .map_err(|error| format!("resolve executable: {error}"));
        }
    }
    Err(format!("executable not found: {command}"))
}

pub fn identity(pid: u32) -> Result<ProcessIdentity, String> {
    let executable = fs::metadata(format!("/proc/{pid}/exe"))
        .map_err(|error| format!("read process executable identity: {error}"))?;
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))
        .map_err(|error| format!("read process start identity: {error}"))?;
    let end = stat
        .rfind(") ")
        .ok_or("invalid process stat identity")?;
    let fields = stat[end + 2..].split_whitespace().collect::<Vec<_>>();
    let start_ticks = fields
        .get(19)
        .ok_or("process start identity missing")?
        .parse::<u64>()
        .map_err(|error| format!("invalid process start identity: {error}"))?;
    Ok(ProcessIdentity {
        pid,
        executable_device: executable.dev(),
        executable_inode: executable.ino(),
        start_ticks,
    })
}

pub fn identity_matches(expected: &ProcessIdentity) -> bool {
    identity(expected.pid).is_ok_and(|actual| actual == *expected)
}

fn signal_group(pid: u32, signal: &str) -> bool {
    let target = format!("-{pid}");
    Command::new("kill")
        .arg(signal)
        .arg("--")
        .arg(&target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
