use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::KubernetesRuntime;

impl KubernetesRuntime {
    fn kubectl(&self) -> Command {
        let mut command = Command::new("kubectl");
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
        deadline: Duration,
    ) -> Result<String, String> {
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
            child
                .stdin
                .as_mut()
                .ok_or("kubectl stdin unavailable")?
                .write_all(payload.as_bytes())
                .map_err(|error| error.to_string())?;
            child.stdin.take();
        }
        let limit = Instant::now() + deadline;
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
            if Instant::now() >= limit {
                let _ = child.kill();
                let _ = child.wait();
                return Err("kubectl deadline elapsed; outcome unknown".to_string());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub(super) fn require_access(&self) -> Result<(), String> {
        let deadline = Duration::from_secs(3);
        self.command(
            &["get", "namespace", &self.config.namespace, "-o", "name"],
            None,
            deadline,
        )?;
        for (verb, resource) in [
            ("get", "pods"),
            ("list", "pods"),
            ("create", "deployments.apps"),
            ("patch", "deployments.apps"),
            ("delete", "deployments.apps"),
            ("create", "services"),
            ("delete", "services"),
            ("create", "persistentvolumeclaims"),
            ("delete", "persistentvolumeclaims"),
        ] {
            let answer = self.command(&["auth", "can-i", verb, resource], None, deadline)?;
            if answer.trim() != "yes" {
                return Err(format!(
                    "kubernetes capability unsupported: {verb} {resource}"
                ));
            }
        }
        Ok(())
    }
}
