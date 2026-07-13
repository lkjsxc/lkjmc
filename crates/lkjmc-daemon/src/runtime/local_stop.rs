use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::local::{LocalRuntime, ProcessEntry};
use super::process;
use super::RuntimeObservation;

impl LocalRuntime {
    pub fn stop(&self, id: &str, timeout: Duration) -> Result<RuntimeObservation, String> {
        let Some(entry) = self.entry(id)? else {
            return Ok(RuntimeObservation::absent("process was not running"));
        };
        let message = self.stop_entry(&entry, timeout)?;
        self.remove_if_same(id, &entry)?;
        Ok(RuntimeObservation::absent(message))
    }

    pub fn shutdown(&self, timeout: Duration) -> Result<(), String> {
        let ids = self.ids()?;
        let deadline = Instant::now() + timeout;
        let mut failures = Vec::new();
        for id in ids {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                failures.push(format!("{id}: shutdown deadline elapsed"));
            } else if let Err(error) = self.stop(&id, remaining) {
                failures.push(format!("{id}: {error}"));
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    }

    fn stop_entry(
        &self,
        entry: &Arc<Mutex<ProcessEntry>>,
        timeout: Duration,
    ) -> Result<&'static str, String> {
        let mut process_entry = entry
            .lock()
            .map_err(|_| "process entry poisoned".to_string())?;
        if !process::identity_matches(&process_entry.identity) {
            return Err("process identity changed; refusing signal".to_string());
        }
        if let Some(stdin) = process_entry
            .child
            .as_mut()
            .and_then(|child| child.stdin.as_mut())
        {
            let _ = stdin.write_all(b"stop\n");
            let _ = stdin.flush();
        }
        let deadline = Instant::now() + timeout;
        let graceful = Instant::now() + (timeout / 2).min(Duration::from_secs(2));
        if self.wait_gone(&mut process_entry, graceful)? {
            return Ok("process stopped from stdin");
        }
        #[cfg(test)]
        if self.take_stop_fault(super::local::StopFault::Signal) {
            return Err("injected TERM signal failure".to_string());
        }
        if !process::identity_matches(&process_entry.identity) {
            return Err("process identity changed before TERM; refusing signal".to_string());
        }
        if !process::terminate_group(process_entry.identity.pid) {
            return Err("send TERM to proved process group failed".to_string());
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if self.wait_gone(&mut process_entry, Instant::now() + remaining / 2)? {
            return Ok("process stopped");
        }
        if !process::kill_group(process_entry.identity.pid) {
            return Err("send KILL to proved process group failed".to_string());
        }
        if self.wait_gone(&mut process_entry, deadline)? {
            return Ok("process killed");
        }
        Err("proved process group remains after KILL deadline".to_string())
    }

    fn wait_gone(&self, entry: &mut ProcessEntry, deadline: Instant) -> Result<bool, String> {
        #[cfg(test)]
        if self.take_stop_fault(super::local::StopFault::Wait) {
            return Err("injected process-group wait failure".to_string());
        }
        while Instant::now() < deadline {
            if let Some(child) = entry.child.as_mut() {
                let _ = child
                    .try_wait()
                    .map_err(|error| format!("reap child: {error}"))?;
            }
            if !process::group_exists(entry.identity.pid) {
                if let Some(child) = entry.child.as_mut() {
                    let _ = child.wait();
                }
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        if let Some(child) = entry.child.as_mut() {
            let _ = child
                .try_wait()
                .map_err(|error| format!("reap child: {error}"))?;
        }
        Ok(!process::group_exists(entry.identity.pid))
    }
}
