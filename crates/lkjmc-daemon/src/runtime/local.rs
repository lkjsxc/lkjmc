use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::runtime::process;
use crate::runtime::RuntimeObservation;

pub struct LocalRuntime {
    pub(super) entries: BTreeMap<String, ProcessEntry>,
    pub(super) fenced: BTreeSet<String>,
    #[cfg(test)]
    stop_fault: Option<StopFault>,
}

pub(super) struct ProcessEntry {
    pub(super) child: Option<Child>,
    pub(super) pid: u32,
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StopFault {
    Signal,
    Wait,
}

impl LocalRuntime {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            fenced: BTreeSet::new(),
            #[cfg(test)]
            stop_fault: None,
        }
    }

    pub fn recover(&mut self, id: &str, pid: u32) -> RuntimeObservation {
        self.entries.remove(id);
        if process::group_exists(pid) {
            self.fenced.insert(id.to_string());
            RuntimeObservation::unhealthy("recovered PID is unverifiable; runtime fenced")
        } else {
            self.fenced.remove(id);
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
        if self.fenced.contains(id) {
            return Err("runtime is fenced after unverifiable PID recovery".to_string());
        }
        if let Some(observation) = self.status(id)? {
            if observation.healthy {
                return Ok(observation);
            }
        }
        let log_dir = Path::new(log_root).join(id);
        fs::create_dir_all(&log_dir).map_err(|error| format!("create log dir: {error}"))?;
        let log_path = log_dir.join("current.log");
        let mut stdout = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)
            .map_err(|error| format!("open log: {error}"))?;
        writeln!(stdout, "lkjmc instance {id}")
            .map_err(|error| format!("write log marker: {error}"))?;
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

    pub fn status(&mut self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        if self.fenced.contains(id) {
            return Ok(Some(RuntimeObservation::unhealthy(
                "recovered PID is fenced",
            )));
        }
        let Some(entry) = self.entries.get_mut(id) else {
            return Ok(None);
        };
        let status = match entry.child.as_mut() {
            Some(child) => match child
                .try_wait()
                .map_err(|error| format!("check process: {error}"))?
            {
                Some(status) if process::group_exists(entry.pid) => {
                    return Ok(Some(RuntimeObservation::unhealthy(format!(
                        "process leader exited with {status}; group remains"
                    ))));
                }
                Some(status) => Some(format!("process exited with {status}")),
                None => None,
            },
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

    #[cfg(test)]
    pub(super) fn inject_stop_fault(&mut self, fault: StopFault) {
        self.stop_fault = Some(fault);
    }

    #[cfg(test)]
    pub(super) fn take_stop_fault(&mut self, fault: StopFault) -> bool {
        if self.stop_fault == Some(fault) {
            self.stop_fault = None;
            true
        } else {
            false
        }
    }
}

impl Default for LocalRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "local_tests.rs"]
mod tests;
