use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::runtime::process;
use crate::runtime::{ProcessIdentity, RuntimeObservation};

pub struct LocalRuntime {
    entries: Mutex<BTreeMap<String, Arc<Mutex<ProcessEntry>>>>,
    #[cfg(test)]
    stop_fault: Mutex<Option<StopFault>>,
}

pub(super) struct ProcessEntry {
    pub(super) child: Option<Child>,
    pub(super) identity: ProcessIdentity,
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
            entries: Mutex::new(BTreeMap::new()),
            #[cfg(test)]
            stop_fault: Mutex::new(None),
        }
    }

    pub fn start(
        &self,
        id: &str,
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        log_root: &str,
        work_dir: &Path,
        deadline: Duration,
    ) -> Result<RuntimeObservation, String> {
        if let Some(observation) = self.status(id)? {
            if observation.healthy {
                return Ok(observation);
            }
        }
        let executable = process::resolve_executable(command)?;
        let expected = fs::metadata(&executable)
            .map_err(|error| format!("stat executable: {error}"))?;
        let log_dir = Path::new(log_root).join(id);
        fs::create_dir_all(&log_dir).map_err(|error| format!("create log dir: {error}"))?;
        let mut stdout = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_dir.join("current.log"))
            .map_err(|error| format!("open log: {error}"))?;
        writeln!(stdout, "lkjmc instance {id}").map_err(|error| format!("write log: {error}"))?;
        let stderr = stdout.try_clone().map_err(|error| format!("clone log: {error}"))?;
        let mut child = Command::new(&executable)
            .args(args)
            .envs(env)
            .current_dir(work_dir)
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|error| format!("spawn process: {error}"))?;
        let pid = child.id();
        let limit = Instant::now() + deadline;
        let identity = loop {
            if let Some(status) = child.try_wait().map_err(|error| format!("check process: {error}"))? {
                self.cleanup_failed_start(pid);
                return Ok(RuntimeObservation::absent(format!("process exited during startup: {status}")));
            }
            if let Ok(identity) = process::identity(pid) {
                if identity.executable_device != expected.dev() || identity.executable_inode != expected.ino() {
                    self.cleanup_failed_start(pid);
                    return Err("spawned executable identity mismatch".to_string());
                }
                break identity;
            }
            if Instant::now() >= limit {
                self.cleanup_failed_start(pid);
                return Err("startup identity deadline elapsed".to_string());
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let entry = Arc::new(Mutex::new(ProcessEntry { child: Some(child), identity: identity.clone() }));
        self.entries.lock().map_err(|_| "process map poisoned".to_string())?
            .insert(id.to_string(), entry);
        Ok(RuntimeObservation::healthy(identity))
    }

    pub fn status(&self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        let Some(entry) = self.entry(id)? else { return Ok(None) };
        let mut entry_guard = entry.lock().map_err(|_| "process entry poisoned".to_string())?;
        if !process::identity_matches(&entry_guard.identity) {
            return Ok(Some(RuntimeObservation::unhealthy("process identity changed; fenced")));
        }
        if let Some(child) = entry_guard.child.as_mut() {
            if let Some(status) = child.try_wait().map_err(|error| format!("check process: {error}"))? {
                let group_remains = process::group_exists(entry_guard.identity.pid);
                drop(entry_guard);
                if group_remains {
                    return Ok(Some(RuntimeObservation::unhealthy(format!("leader exited with {status}; proved group remains"))));
                }
                self.remove_if_same(id, &entry)?;
                return Ok(Some(RuntimeObservation::absent(format!("process exited with {status}"))));
            }
        }
        Ok(Some(RuntimeObservation::healthy(entry_guard.identity.clone())))
    }

    #[cfg(test)]
    pub fn recover(&self, id: &str, identity: ProcessIdentity) -> RuntimeObservation {
        let matches = process::identity_matches(&identity);
        let entry = Arc::new(Mutex::new(ProcessEntry { child: None, identity: identity.clone() }));
        match self.entries.lock() {
            Ok(mut entries) => {
                entries.insert(id.to_string(), entry);
                if matches {
                    RuntimeObservation::healthy(identity)
                } else {
                    RuntimeObservation::unhealthy("persisted process identity is absent or changed; fenced")
                }
            }
            Err(_) => RuntimeObservation::unhealthy("process map poisoned during recovery"),
        }
    }

    pub(super) fn entry(&self, id: &str) -> Result<Option<Arc<Mutex<ProcessEntry>>>, String> {
        self.entries.lock().map(|entries| entries.get(id).cloned())
            .map_err(|_| "process map poisoned".to_string())
    }

    pub(super) fn remove_if_same(&self, id: &str, entry: &Arc<Mutex<ProcessEntry>>) -> Result<(), String> {
        let mut entries = self.entries.lock().map_err(|_| "process map poisoned".to_string())?;
        if entries.get(id).is_some_and(|current| Arc::ptr_eq(current, entry)) { entries.remove(id); }
        Ok(())
    }

    pub(super) fn ids(&self) -> Result<Vec<String>, String> {
        self.entries.lock().map(|entries| entries.keys().cloned().collect())
            .map_err(|_| "process map poisoned".to_string())
    }

    fn cleanup_failed_start(&self, pid: u32) {
        if process::group_exists(pid) { let _ = process::kill_group(pid); }
    }

    #[cfg(test)]
    pub(super) fn inject_stop_fault(&self, fault: StopFault) {
        if let Ok(mut value) = self.stop_fault.lock() { *value = Some(fault); }
    }

    #[cfg(test)]
    pub(super) fn take_stop_fault(&self, fault: StopFault) -> bool {
        self.stop_fault.lock().is_ok_and(|mut value| {
            if *value == Some(fault) { *value = None; true } else { false }
        })
    }
}

impl Default for LocalRuntime { fn default() -> Self { Self::new() } }

#[cfg(test)]
#[path = "local_tests.rs"]
mod tests;
