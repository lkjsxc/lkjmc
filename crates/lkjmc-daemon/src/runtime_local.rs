use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::process;
use crate::runtime::RuntimeObservation;

pub struct LocalRuntime {
    entries: BTreeMap<String, ProcessEntry>,
}

struct ProcessEntry {
    child: Child,
    pid: u32,
}

impl LocalRuntime {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn start(
        &mut self,
        id: &str,
        command: &str,
        args: &[String],
        log_root: &str,
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
        let mut child_command = Command::new("setsid");
        child_command.arg("--wait").arg(command).args(args);
        let child = child_command
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|error| format!("spawn process: {error}"))?;
        let pid = child.id();
        self.entries
            .insert(id.to_string(), ProcessEntry { child, pid });
        Ok(RuntimeObservation::healthy(pid))
    }

    pub fn stop(&mut self, id: &str, timeout: Duration) -> Result<RuntimeObservation, String> {
        let Some(mut entry) = self.entries.remove(id) else {
            return Ok(RuntimeObservation::absent("process was not running"));
        };
        process::terminate_group(entry.pid);
        let deadline = Instant::now() + timeout;
        loop {
            match entry.child.try_wait() {
                Ok(Some(_status)) => return Ok(RuntimeObservation::absent("process stopped")),
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(100))
                }
                Ok(None) => break,
                Err(error) => return Err(format!("wait for process: {error}")),
            }
        }
        process::kill_group(entry.pid);
        entry
            .child
            .wait()
            .map_err(|error| format!("wait after kill: {error}"))?;
        Ok(RuntimeObservation::absent("process killed"))
    }

    pub fn status(&mut self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        let Some(entry) = self.entries.get_mut(id) else {
            return Ok(None);
        };
        match entry.child.try_wait() {
            Ok(Some(status)) => {
                let message = format!("process exited with {status}");
                self.entries.remove(id);
                Ok(Some(RuntimeObservation::absent(message)))
            }
            Ok(None) => Ok(Some(RuntimeObservation::healthy(entry.pid))),
            Err(error) => Err(format!("check process: {error}")),
        }
    }

    pub fn is_running(&mut self, id: &str) -> Result<bool, String> {
        Ok(self.status(id)?.is_some_and(|status| status.healthy))
    }
}

impl Default for LocalRuntime {
    fn default() -> Self {
        Self::new()
    }
}
