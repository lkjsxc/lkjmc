use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::KubernetesRuntime;

pub(super) struct CommandDeadline {
    end: Instant,
}

impl CommandDeadline {
    pub(super) fn new(total: Duration) -> Self {
        Self {
            end: Instant::now() + total,
        }
    }

    fn remaining(&self) -> Result<Duration, String> {
        self.end
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| "kubectl deadline elapsed; outcome unknown".to_string())
    }
}

impl KubernetesRuntime {
    fn kubectl(&self) -> Command {
        let mut command = Command::new(&self.kubectl_program);
        command.arg("-n").arg(&self.config.namespace);
        if let Some(path) = &self.config.kubeconfig_path {
            command.arg("--kubeconfig").arg(path);
        }
        command
    }

    pub(super) fn command(
        &self,
        args: &[&str],
        input: Option<&str>,
        deadline: &CommandDeadline,
    ) -> Result<String, String> {
        deadline.remaining()?;
        let mut child = self
            .kubectl()
            .args(args)
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("kubectl unavailable: {error}"))?;
        if let Some(payload) = input {
            deadline.remaining()?;
            child
                .stdin
                .as_mut()
                .ok_or("kubectl stdin unavailable")?
                .write_all(payload.as_bytes())
                .map_err(|error| error.to_string())?;
            child.stdin.take();
        }
        loop {
            if child
                .try_wait()
                .map_err(|error| error.to_string())?
                .is_some()
            {
                let output = child
                    .wait_with_output()
                    .map_err(|error| error.to_string())?;
                if !output.status.success() {
                    return Err(format!(
                        "kubectl failed: {}",
                        String::from_utf8_lossy(&output.stderr).trim()
                    ));
                }
                return String::from_utf8(output.stdout).map_err(|error| error.to_string());
            }
            let remaining = match deadline.remaining() {
                Ok(remaining) => remaining,
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error);
                }
            };
            std::thread::sleep(remaining.min(Duration::from_millis(20)));
        }
    }

    pub(crate) fn require_access(&self, total: Duration) -> Result<(), String> {
        let deadline = CommandDeadline::new(total);
        self.command(
            &["get", "namespace", &self.config.namespace, "-o", "name"],
            None,
            &deadline,
        )?;
        for (verb, resource) in [("get", "pods"), ("list", "pods"), ("get", "pods/log")] {
            let answer = self.command(&["auth", "can-i", verb, resource], None, &deadline)?;
            if answer.trim() != "yes" {
                return Err(format!(
                    "kubernetes capability unsupported: {verb} {resource}"
                ));
            }
        }
        Ok(())
    }
}
