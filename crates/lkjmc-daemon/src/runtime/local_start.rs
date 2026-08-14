use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::local::{LocalRuntime, ProcessEntry};
use super::{local_identity, process, RuntimeObservation};

impl LocalRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn runtime_start(
        &self,
        id: &str,
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        log_root: &str,
        work_dir: &Path,
        deadline: Duration,
    ) -> Result<RuntimeObservation, String> {
        if let Some(observation) = self.runtime_status(id)? {
            if observation.healthy {
                return Ok(observation);
            }
            return Err(observation
                .message
                .unwrap_or_else(|| "existing runtime identity is unhealthy".to_string()));
        }
        let executable = process::resolve_executable(command)?;
        let expected =
            fs::metadata(&executable).map_err(|error| format!("stat executable: {error}"))?;
        let log_dir = Path::new(log_root).join(id);
        fs::create_dir_all(&log_dir).map_err(|error| format!("create log dir: {error}"))?;
        let mut stdout = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_dir.join("current.log"))
            .map_err(|error| format!("open log: {error}"))?;
        writeln!(stdout, "lkjmc instance {id}").map_err(|error| format!("write log: {error}"))?;
        let stderr = stdout
            .try_clone()
            .map_err(|error| format!("clone log: {error}"))?;
        let mut child = Command::new(&executable)
            .env_clear()
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
            if let Some(status) = child
                .try_wait()
                .map_err(|error| format!("check process: {error}"))?
            {
                self.cleanup_failed_start(pid);
                return Ok(RuntimeObservation::absent(format!(
                    "process exited during startup: {status}"
                )));
            }
            if let Ok(identity) = process::identity(pid) {
                if identity.executable_device == expected.dev()
                    && identity.executable_inode == expected.ino()
                {
                    break identity;
                }
            }
            if Instant::now() >= limit {
                self.cleanup_failed_start(pid);
                let _ = child.wait();
                return Err("startup executable identity deadline elapsed".to_string());
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let stability_limit = limit.min(Instant::now() + Duration::from_millis(20));
        while Instant::now() < stability_limit {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| format!("check process: {error}"))?
            {
                self.cleanup_failed_start(pid);
                return Ok(RuntimeObservation::absent(format!(
                    "process exited during startup: {status}"
                )));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        if let Err(error) = local_identity::write(work_dir, &identity) {
            self.cleanup_failed_start(pid);
            let _ = child.wait();
            return Err(error);
        }
        match self.entries.lock() {
            Ok(mut entries) => {
                let entry = Arc::new(Mutex::new(ProcessEntry {
                    child: Some(child),
                    identity: identity.clone(),
                    work_dir: work_dir.to_path_buf(),
                }));
                entries.insert(id.to_string(), entry);
                Ok(RuntimeObservation::healthy(identity))
            }
            Err(_) => {
                self.cleanup_failed_start(pid);
                let _ = child.wait();
                let _ = local_identity::remove_from(work_dir);
                Err("process map poisoned".to_string())
            }
        }
    }
}
