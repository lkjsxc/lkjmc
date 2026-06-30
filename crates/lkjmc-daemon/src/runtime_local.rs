use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::process;
use crate::runtime::RuntimeObservation;

pub struct LocalRuntime {
    entries: BTreeMap<String, ProcessEntry>,
}

struct ProcessEntry {
    child: Option<Child>,
    pid: u32,
}

impl LocalRuntime {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn recover(&mut self, id: &str, pid: u32) -> RuntimeObservation {
        if process::group_exists(pid) {
            self.entries
                .insert(id.to_string(), ProcessEntry { child: None, pid });
            RuntimeObservation::healthy(pid)
        } else {
            self.entries.remove(id);
            RuntimeObservation::absent("process missing after daemon restart")
        }
    }

    pub fn start(
        &mut self,
        id: &str,
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        log_root: &str,
        work_dir: &Path,
    ) -> Result<RuntimeObservation, String> {
        if let Some(observation) = self.status(id)? {
            if observation.healthy {
                return Ok(observation);
            }
        }
        let log_dir = Path::new(log_root).join(id);
        fs::create_dir_all(&log_dir).map_err(|error| format!("create log dir: {error}"))?;
        let log_path = log_dir.join("current.log");
        let stdout = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|error| format!("open log: {error}"))?;
        let stderr = stdout
            .try_clone()
            .map_err(|error| format!("clone log: {error}"))?;
        let mut child_command = Command::new(command);
        child_command
            .args(args)
            .envs(env)
            .current_dir(work_dir)
            .process_group(0);
        let mut child = child_command
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|error| format!("spawn process: {error}"))?;
        let pid = child.id();
        std::thread::sleep(Duration::from_millis(500));
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("check process: {error}"))?
        {
            return Ok(RuntimeObservation::absent(format!(
                "process exited immediately with {status}"
            )));
        }
        self.entries.insert(
            id.to_string(),
            ProcessEntry {
                child: Some(child),
                pid,
            },
        );
        Ok(RuntimeObservation::healthy(pid))
    }

    pub fn stop(&mut self, id: &str, timeout: Duration) -> Result<RuntimeObservation, String> {
        let Some(mut entry) = self.entries.remove(id) else {
            return Ok(RuntimeObservation::absent("process was not running"));
        };
        let graceful_deadline = Instant::now() + timeout.min(Duration::from_secs(2));
        if let Some(child) = entry.child.as_mut() {
            write_stop(child);
            if wait_child(child, graceful_deadline)? {
                return Ok(RuntimeObservation::absent("process stopped from stdin"));
            }
        }
        process::terminate_group(entry.pid);
        let deadline = Instant::now() + timeout;
        if let Some(child) = entry.child.as_mut() {
            if wait_child(child, deadline)? {
                return Ok(RuntimeObservation::absent("process stopped"));
            }
        } else if wait_group_gone(entry.pid, deadline) {
            return Ok(RuntimeObservation::absent("process stopped"));
        }
        process::kill_group(entry.pid);
        if let Some(mut child) = entry.child {
            child
                .wait()
                .map_err(|error| format!("wait after kill: {error}"))?;
        }
        Ok(RuntimeObservation::absent("process killed"))
    }

    pub fn status(&mut self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        let Some(entry) = self.entries.get_mut(id) else {
            return Ok(None);
        };
        let status = match entry.child.as_mut() {
            Some(child) => child
                .try_wait()
                .map_err(|error| format!("check process: {error}"))?
                .map(|status| format!("process exited with {status}")),
            None if process::group_exists(entry.pid) => None,
            None => Some("process missing after daemon restart".to_string()),
        };
        match status {
            Some(message) => {
                self.entries.remove(id);
                Ok(Some(RuntimeObservation::absent(message)))
            }
            None => Ok(Some(RuntimeObservation::healthy(entry.pid))),
        }
    }
}

impl Default for LocalRuntime {
    fn default() -> Self {
        Self::new()
    }
}

fn write_stop(child: &mut Child) {
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(b"stop\n");
        let _ = stdin.flush();
    }
}

fn wait_child(child: &mut Child, deadline: Instant) -> Result<bool, String> {
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => return Ok(true),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)),
            Ok(None) => return Ok(false),
            Err(error) => return Err(format!("wait for process: {error}")),
        }
    }
}

fn wait_group_gone(pid: u32, deadline: Instant) -> bool {
    while Instant::now() < deadline {
        if !process::group_exists(pid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}
