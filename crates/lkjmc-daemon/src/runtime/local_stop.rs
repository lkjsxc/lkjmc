use std::io::Write;
use std::time::{Duration, Instant};

use super::local::LocalRuntime;
use super::process;
use super::RuntimeObservation;

impl LocalRuntime {
    pub fn stop(&mut self, id: &str, timeout: Duration) -> Result<RuntimeObservation, String> {
        if self.fenced.contains(id) {
            return Err("refusing to signal unverifiable recovered PID".to_string());
        }
        if !self.entries.contains_key(id) {
            return Ok(RuntimeObservation::absent("process was not running"));
        }
        let pid = self.request_stop(id)?;
        let graceful_deadline = Instant::now() + timeout.min(Duration::from_secs(2));
        if self.wait_group_gone(pid, graceful_deadline)? {
            return Ok(self.record_absence(id, "process stopped from stdin"));
        }
        self.send_term(pid)?;
        if self.wait_group_gone(pid, Instant::now() + timeout)? {
            return Ok(self.record_absence(id, "process stopped"));
        }
        self.send_kill(pid)?;
        if self.wait_group_gone(pid, Instant::now() + timeout)? {
            return Ok(self.record_absence(id, "process killed"));
        }
        Err("process group remains after KILL".to_string())
    }

    fn request_stop(&mut self, id: &str) -> Result<u32, String> {
        let entry = self
            .entries
            .get_mut(id)
            .ok_or_else(|| "process was not running".to_string())?;
        if let Some(child) = entry.child.as_mut() {
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(b"stop\n");
                let _ = stdin.flush();
            }
        }
        Ok(entry.pid)
    }

    fn record_absence(&mut self, id: &str, message: &str) -> RuntimeObservation {
        self.entries.remove(id);
        RuntimeObservation::absent(message)
    }

    fn send_term(&mut self, pid: u32) -> Result<(), String> {
        #[cfg(test)]
        if self.take_stop_fault(super::local::StopFault::Signal) {
            return Err("injected TERM signal failure".to_string());
        }
        if process::terminate_group(pid) {
            Ok(())
        } else {
            Err("send TERM to process group failed".to_string())
        }
    }

    fn send_kill(&mut self, pid: u32) -> Result<(), String> {
        if process::kill_group(pid) {
            Ok(())
        } else {
            Err("send KILL to process group failed".to_string())
        }
    }

    fn wait_group_gone(&mut self, pid: u32, deadline: Instant) -> Result<bool, String> {
        #[cfg(test)]
        if self.take_stop_fault(super::local::StopFault::Wait) {
            return Err("injected process-group wait failure".to_string());
        }
        while Instant::now() < deadline {
            self.reap_child(pid)?;
            if !process::group_exists(pid) {
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        self.reap_child(pid)?;
        Ok(!process::group_exists(pid))
    }

    fn reap_child(&mut self, pid: u32) -> Result<(), String> {
        let Some(child) = self
            .entries
            .values_mut()
            .find(|entry| entry.pid == pid)
            .and_then(|entry| entry.child.as_mut())
        else {
            return Ok(());
        };
        child
            .try_wait()
            .map(|_| ())
            .map_err(|error| format!("wait for process group: {error}"))
    }
}
