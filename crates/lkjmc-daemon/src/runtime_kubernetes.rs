use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use lkjmc_core::config::KubernetesRuntimeConfig;
use lkjmc_core::kubernetes::{self, KubernetesPlanInput};

use crate::runtime::{RuntimeAdapter, RuntimeCapabilities, RuntimeObservation};

pub struct KubernetesRuntime {
    config: KubernetesRuntimeConfig,
}

impl KubernetesRuntime {
    pub fn new(config: KubernetesRuntimeConfig) -> Self {
        Self { config }
    }

    fn kubectl(&self) -> Command {
        let mut command = Command::new("kubectl");
        command.arg("-n").arg(&self.config.namespace);
        if let Some(path) = &self.config.kubeconfig_path {
            command.arg("--kubeconfig").arg(path);
        }
        command
    }

    fn apply(&self, payload: &str) -> Result<(), String> {
        let mut child = self
            .kubectl()
            .arg("apply")
            .arg("-f")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("kubectl apply: {error}"))?;
        child
            .stdin
            .as_mut()
            .ok_or("kubectl stdin unavailable")?
            .write_all(payload.as_bytes())
            .map_err(|error| error.to_string())?;
        let output = child
            .wait_with_output()
            .map_err(|error| error.to_string())?;
        status(output.status.success(), &output.stderr)
    }

    fn command(&self, args: &[&str]) -> Result<String, String> {
        let output = self
            .kubectl()
            .args(args)
            .output()
            .map_err(|error| error.to_string())?;
        status(output.status.success(), &output.stderr)?;
        String::from_utf8(output.stdout).map_err(|error| error.to_string())
    }
}

impl RuntimeAdapter for KubernetesRuntime {
    fn name(&self) -> &'static str {
        "kubernetes"
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities::kubernetes()
    }

    fn recover(&mut self, id: &str, _pid: u32) -> RuntimeObservation {
        self.status(id)
            .ok()
            .flatten()
            .unwrap_or_else(|| RuntimeObservation::absent("kubernetes object absent"))
    }

    fn start(
        &mut self,
        id: &str,
        _command: &str,
        _args: &[String],
        _env: &BTreeMap<String, String>,
        _log_root: &str,
        _work_dir: &Path,
    ) -> Result<RuntimeObservation, String> {
        let input = KubernetesPlanInput {
            namespace: &self.config.namespace,
            instance_id: id,
            implementation: "minecraft",
            image: &self.config.server_image,
            service_type: &self.config.service_type,
            storage_class: &self.config.storage_class,
            storage_size: &self.config.storage_size,
            server_port: 25565,
            cpu_request: &self.config.cpu_request,
            memory_request: &self.config.memory_request,
        };
        self.apply(&kubernetes::object_list(&input).to_string())?;
        self.status(id).map(|value| {
            value.unwrap_or_else(|| {
                RuntimeObservation::unhealthy("kubernetes objects applied but not observed")
            })
        })
    }

    fn stop(&mut self, id: &str, _timeout: Duration) -> Result<RuntimeObservation, String> {
        let name = format!("deployment/lkjmc-{id}");
        self.command(&["scale", &name, "--replicas=0"])?;
        Ok(RuntimeObservation::absent(
            "kubernetes workload scaled to zero",
        ))
    }

    fn status(&mut self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        let selector = kubernetes::selector(id);
        let output = self.command(&["get", "pods", "-l", &selector, "-o", "json"])?;
        let Some(observation) = kubernetes::observe_pods_json(&output)? else {
            return Ok(None);
        };
        Ok(Some(if observation.ready {
            RuntimeObservation {
                observed_state: "kubernetes-ready".into(),
                healthy: true,
                pid: None,
                message: Some(format!(
                    "ready pod observed; restarts={}",
                    observation.restart_count
                )),
            }
        } else {
            let reason = observation
                .last_error
                .or(observation.phase)
                .unwrap_or_else(|| "not ready".into());
            RuntimeObservation::unhealthy(&format!("kubernetes pod {reason}"))
        }))
    }

    fn logs(&mut self, id: &str, _log_root: &str, lines: usize) -> Result<Vec<String>, String> {
        let selector = kubernetes::selector(id);
        let text = self.command(&["logs", "-l", &selector, "--tail", &lines.to_string()])?;
        Ok(text.lines().map(ToString::to_string).collect())
    }

    fn delete(&mut self, id: &str) -> Result<RuntimeObservation, String> {
        let selector = kubernetes::selector(id);
        self.command(&[
            "delete",
            "deployment,service,pvc",
            "-l",
            &selector,
            "--ignore-not-found=true",
        ])?;
        Ok(RuntimeObservation::absent(
            "kubernetes owned objects deleted",
        ))
    }
}

fn status(success: bool, stderr: &[u8]) -> Result<(), String> {
    success
        .then_some(())
        .ok_or_else(|| String::from_utf8_lossy(stderr).to_string())
}
