use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{OpsError, Result};
use crate::manifest::sha256_file;

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub stdin: Vec<u8>,
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn run_bounded(spec: &CommandSpec) -> Result<CommandOutput> {
    validate_spec(spec)?;
    let executable = trusted_executable(&spec.executable)?;
    run_prevalidated(spec, &executable)
}

fn validate_spec(spec: &CommandSpec) -> Result<()> {
    if spec.timeout.is_zero() || spec.timeout > Duration::from_secs(3600) {
        return Err(OpsError::message(
            "command timeout is outside 1..=3600 seconds",
        ));
    }
    if spec.max_output_bytes == 0 || spec.max_output_bytes > 16 * 1024 * 1024 {
        return Err(OpsError::message(
            "command output bound is outside 1..=16777216 bytes",
        ));
    }
    if spec.stdin.len() > 16 * 1024 * 1024 {
        return Err(OpsError::message("command input exceeds 16777216 bytes"));
    }
    if spec.arguments.len() > 256
        || spec.arguments.iter().map(String::len).sum::<usize>() > 1024 * 1024
    {
        return Err(OpsError::message("command arguments exceed their bound"));
    }
    if spec.environment.len() > 64
        || spec
            .environment
            .iter()
            .map(|(key, value)| key.len() + value.len())
            .sum::<usize>()
            > 1024 * 1024
    {
        return Err(OpsError::message("command environment exceeds its bound"));
    }
    Ok(())
}

pub fn run_bounded_owned(
    spec: &CommandSpec,
    expected_uid: u32,
    expected_sha256: Option<&str>,
) -> Result<CommandOutput> {
    validate_spec(spec)?;
    let executable = owned_executable(&spec.executable, expected_uid, expected_sha256)?;
    run_prevalidated(spec, &executable)
}

fn run_prevalidated(spec: &CommandSpec, executable: &Path) -> Result<CommandOutput> {
    let mut command = Command::new(executable);
    command
        .args(&spec.arguments)
        .env_clear()
        .env(
            "PATH",
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        )
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .envs(&spec.environment)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let mut child = command
        .spawn()
        .map_err(|error| OpsError::context("cannot execute trusted command", error))?;
    if let Some(mut input) = child.stdin.take() {
        input
            .write_all(&spec.stdin)
            .map_err(|error| OpsError::context("cannot write command input", error))?;
    }
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| OpsError::message("command stdout pipe is unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| OpsError::message("command stderr pipe is unavailable"))?;
    let max_stdout = spec.max_output_bytes;
    let max_stderr = spec.max_output_bytes;
    let stdout_reader = thread::spawn(move || read_bounded(stdout, max_stdout));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, max_stderr));
    let deadline = Instant::now() + spec.timeout;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| OpsError::context("cannot poll trusted command", error))?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let process_group = rustix::process::Pid::from_child(&child);
            let _ =
                rustix::process::kill_process_group(process_group, rustix::process::Signal::KILL);
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(OpsError::message(format!(
                "trusted command timed out: {}",
                command_label(executable)
            )));
        }
        thread::sleep(Duration::from_millis(20));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| OpsError::message("command stdout reader failed"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| OpsError::message("command stderr reader failed"))??;
    if stdout.truncated || stderr.truncated {
        return Err(OpsError::message(format!(
            "trusted command exceeded its output bound: {}",
            command_label(executable)
        )));
    }
    let code = status.code().unwrap_or(-1);
    Ok(CommandOutput {
        status: code,
        stdout: stdout.bytes,
        stderr: stderr.bytes,
    })
}

pub fn owned_executable(
    path: &Path,
    expected_uid: u32,
    expected_sha256: Option<&str>,
) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(OpsError::message("owned executable path must be absolute"));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| OpsError::context("cannot inspect owned executable", error))?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.uid() != expected_uid
        || metadata.mode() & 0o022 != 0
        || metadata.mode() & 0o111 == 0
    {
        return Err(OpsError::message(format!(
            "owned executable identity or mode is unsafe: {}",
            path.display()
        )));
    }
    if let Some(expected) = expected_sha256 {
        if sha256_file(path)? != expected {
            return Err(OpsError::message(format!(
                "owned executable digest differs: {}",
                path.display()
            )));
        }
    }
    Ok(path.to_path_buf())
}

pub fn require_success(output: CommandOutput, label: &str) -> Result<CommandOutput> {
    if output.status == 0 {
        Ok(output)
    } else {
        let detail = last_line(&output.stderr)
            .or_else(|| last_line(&output.stdout))
            .unwrap_or("no diagnostic output");
        Err(OpsError::message(format!(
            "{label} failed with status {}: {detail}",
            output.status
        )))
    }
}

pub fn trusted_executable(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(OpsError::message(
            "trusted executable path must be absolute",
        ));
    }
    let allowed = allowed_roots(path).ok_or_else(|| {
        OpsError::message(format!("unexpected external command: {}", path.display()))
    })?;
    let resolved = fs::canonicalize(path)
        .map_err(|error| OpsError::context("cannot resolve trusted executable", error))?;
    if !allowed.iter().any(|root| resolved.starts_with(root)) {
        return Err(OpsError::message(format!(
            "trusted executable resolves outside its allowed roots: {}",
            resolved.display()
        )));
    }
    validate_root_ancestry(&resolved)?;
    let metadata = fs::symlink_metadata(&resolved)
        .map_err(|error| OpsError::context("cannot inspect trusted executable", error))?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.uid() != 0
        || metadata.mode() & 0o022 != 0
        || metadata.mode() & 0o111 == 0
    {
        return Err(OpsError::message(format!(
            "trusted executable identity or mode is unsafe: {}",
            resolved.display()
        )));
    }
    Ok(resolved)
}

fn allowed_roots(path: &Path) -> Option<Vec<&'static Path>> {
    let name = path.file_name()?.to_str()?;
    match name {
        "systemctl" | "psql" | "pg_dump" | "pg_restore" | "pgrep" | "getent" | "df" => Some(vec![
            Path::new("/usr/bin"),
            Path::new("/usr/lib/postgresql"),
            Path::new("/usr/share/postgresql-common"),
        ]),
        "runuser" | "useradd" | "groupadd" => Some(vec![Path::new("/usr/sbin")]),
        "java" => Some(vec![Path::new("/usr/bin"), Path::new("/usr/lib/jvm")]),
        _ => None,
    }
}

fn validate_root_ancestry(path: &Path) -> Result<()> {
    let mut current = PathBuf::from("/");
    for component in path.components().skip(1) {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| OpsError::context("cannot inspect command ancestry", error))?;
        if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
            return Err(OpsError::message(format!(
                "trusted command ancestry is writable or not root-owned: {}",
                current.display()
            )));
        }
        if current != path && !metadata.file_type().is_dir() {
            return Err(OpsError::message(format!(
                "trusted command ancestry is not a directory: {}",
                current.display()
            )));
        }
    }
    Ok(())
}

#[derive(Debug)]
struct BoundedRead {
    bytes: Vec<u8>,
    truncated: bool,
}

fn read_bounded(mut input: impl Read, limit: usize) -> Result<BoundedRead> {
    let mut bytes = Vec::with_capacity(limit.min(64 * 1024));
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    loop {
        let count = input
            .read(&mut buffer)
            .map_err(|error| OpsError::context("cannot read command output", error))?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(bytes.len());
        let retained = remaining.min(count);
        bytes.extend_from_slice(&buffer[..retained]);
        if retained < count {
            truncated = true;
        }
    }
    Ok(BoundedRead { bytes, truncated })
}

fn last_line(bytes: &[u8]) -> Option<&str> {
    let text = std::str::from_utf8(bytes).ok()?;
    text.lines().rev().find(|line| !line.trim().is_empty())
}

fn command_label(path: &Path) -> &str {
    match path.file_name().and_then(|value| value.to_str()) {
        Some(value) => value,
        None => "external-command",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::*;

    #[test]
    fn unapproved_command_is_rejected_before_execution() {
        let spec = CommandSpec {
            executable: PathBuf::from("/usr/bin/true"),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            stdin: Vec::new(),
            timeout: Duration::from_secs(1),
            max_output_bytes: 1024,
        };
        let error = run_bounded(&spec)
            .err()
            .map(|value| value.to_string())
            .unwrap_or_default();
        assert!(error.contains("unexpected external command"));
    }

    #[test]
    fn owned_rust_process_is_executed_with_bounded_output() -> Result<()> {
        let executable = std::env::current_exe()
            .map_err(|error| OpsError::context("cannot locate process test binary", error))?;
        let digest = sha256_file(&executable)?;
        let output = run_bounded_owned(
            &CommandSpec {
                executable,
                arguments: vec!["--list".to_string()],
                environment: BTreeMap::new(),
                stdin: Vec::new(),
                timeout: Duration::from_secs(10),
                max_output_bytes: 1024 * 1024,
            },
            crate::secure_fs::effective_uid(),
            Some(&digest),
        )?;
        let output = require_success(output, "owned Rust process fixture")?;
        let stdout = std::str::from_utf8(&output.stdout)
            .map_err(|error| OpsError::context("process fixture output is not UTF-8", error))?;
        if !stdout.contains("owned_rust_process_is_executed_with_bounded_output") {
            return Err(OpsError::message(
                "owned Rust process fixture did not independently list its test",
            ));
        }
        Ok(())
    }

    #[test]
    fn timed_out_owned_process_group_does_not_leave_its_descendant_running() -> Result<()> {
        let executable = std::env::current_exe()
            .map_err(|error| OpsError::context("cannot locate process test binary", error))?;
        let pid_path = std::env::temp_dir().join(format!(
            "lkjmc-ops-process-descendant-{}.pid",
            Uuid::new_v4()
        ));
        let error = run_bounded_owned(
            &CommandSpec {
                executable,
                arguments: vec![
                    "--exact".to_string(),
                    "process::tests::timeout_parent_fixture".to_string(),
                    "--nocapture".to_string(),
                ],
                environment: BTreeMap::from([(
                    "LKJMC_OPS_TIMEOUT_PID_FILE".to_string(),
                    pid_path.display().to_string(),
                )]),
                stdin: Vec::new(),
                timeout: Duration::from_millis(500),
                max_output_bytes: 1024 * 1024,
            },
            crate::secure_fs::effective_uid(),
            None,
        )
        .err()
        .ok_or_else(|| OpsError::message("timed process fixture unexpectedly passed"))?;
        assert!(error.to_string().contains("timed out"));
        let pid_text = fs::read_to_string(&pid_path)
            .map_err(|error| OpsError::context("descendant PID was not recorded", error))?;
        let pid = pid_text
            .trim()
            .parse::<u32>()
            .map_err(|error| OpsError::context("recorded descendant PID is invalid", error))?;
        let deadline = Instant::now() + Duration::from_secs(2);
        while process_is_running(pid)? && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
        }
        let _ = fs::remove_file(&pid_path);
        assert!(!process_is_running(pid)?);
        Ok(())
    }

    #[test]
    fn timeout_parent_fixture() -> Result<()> {
        let Ok(pid_path) = std::env::var("LKJMC_OPS_TIMEOUT_PID_FILE") else {
            return Ok(());
        };
        let executable = std::env::current_exe()
            .map_err(|error| OpsError::context("cannot locate descendant fixture", error))?;
        let child = Command::new(executable)
            .args([
                "--exact",
                "process::tests::timeout_descendant_fixture",
                "--nocapture",
            ])
            .env("LKJMC_OPS_TIMEOUT_DESCENDANT", "1")
            .spawn()
            .map_err(|error| OpsError::context("cannot spawn descendant fixture", error))?;
        fs::write(pid_path, format!("{}\n", child.id()))
            .map_err(|error| OpsError::context("cannot record descendant fixture PID", error))?;
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }

    #[test]
    fn timeout_descendant_fixture() {
        if std::env::var_os("LKJMC_OPS_TIMEOUT_DESCENDANT").is_none() {
            return;
        }
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }

    fn process_is_running(pid: u32) -> Result<bool> {
        match fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(status) => Ok(status
                .split_once(") ")
                .and_then(|(_, suffix)| suffix.chars().next())
                .is_some_and(|state| state != 'Z')),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(OpsError::context(
                "cannot inspect descendant fixture process",
                error,
            )),
        }
    }
}
