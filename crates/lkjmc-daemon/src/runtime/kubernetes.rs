use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use lkjmc_core::config::KubernetesRuntimeConfig;
use lkjmc_core::kubernetes::{self, KubernetesPlanInput};

use crate::runtime::adapter::{require, RuntimeRequirements};
use crate::runtime::{RuntimeAdapter, RuntimeCapabilities, RuntimeObservation};

pub struct KubernetesRuntime { config: KubernetesRuntimeConfig }

impl KubernetesRuntime {
    pub fn new(config: KubernetesRuntimeConfig) -> Self { Self { config } }

    fn kubectl(&self) -> Command {
        let mut command = Command::new("kubectl");
        command.arg("-n").arg(&self.config.namespace);
        if let Some(path) = &self.config.kubeconfig_path {
            command.arg("--kubeconfig").arg(path);
        }
        command
    }

    fn command(&self, args: &[&str], input: Option<&str>, deadline: Duration) -> Result<String, String> {
        let mut child = self.kubectl().args(args)
            .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
            .map_err(|error| format!("kubectl unavailable: {error}"))?;
        if let Some(payload) = input {
            child.stdin.as_mut().ok_or("kubectl stdin unavailable")?
                .write_all(payload.as_bytes()).map_err(|error| error.to_string())?;
            child.stdin.take();
        }
        let limit = Instant::now() + deadline;
        loop {
            if child.try_wait().map_err(|error| error.to_string())?.is_some() {
                let output = child.wait_with_output().map_err(|error| error.to_string())?;
                if !output.status.success() {
                    return Err(format!("kubectl failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
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

    fn require_access(&self) -> Result<(), String> {
        let deadline = Duration::from_secs(3);
        self.command(&["get", "namespace", &self.config.namespace, "-o", "name"], None, deadline)?;
        for (verb, resource) in [
            ("get", "pods"), ("list", "pods"), ("create", "deployments.apps"),
            ("patch", "deployments.apps"), ("delete", "deployments.apps"),
            ("create", "services"), ("delete", "services"),
            ("create", "persistentvolumeclaims"), ("delete", "persistentvolumeclaims"),
        ] {
            let answer = self.command(&["auth", "can-i", verb, resource], None, deadline)?;
            if answer.trim() != "yes" { return Err(format!("kubernetes capability unsupported: {verb} {resource}")); }
        }
        Ok(())
    }
}

impl RuntimeAdapter for KubernetesRuntime {
    fn name(&self) -> &'static str { "kubernetes" }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            process_identity: false, readiness: true, storage: true, secrets: false,
            configuration: false, logs: true, recovery: true,
        }
    }

    fn check_capabilities(&self) -> Result<(), String> {
        require(self.capabilities(), RuntimeRequirements {
            readiness: true, storage: true, logs: true, recovery: true,
            ..RuntimeRequirements::default()
        })?;
        self.require_access()
    }

    fn start(
        &self, id: &str, command: &str, args: &[String], env: &BTreeMap<String, String>,
        _log_root: &str, work_dir: &Path, deadline: Duration,
    ) -> Result<RuntimeObservation, String> {
        self.require_access()?;
        let server_port = env.get("LKJMC_SERVER_PORT").and_then(|value| value.parse().ok())
            .ok_or("kubernetes launch requires proved server port")?;
        let implementation = env.get("LKJMC_INSTANCE_KIND").cloned()
            .ok_or("kubernetes launch requires implementation kind")?;
        let input = KubernetesPlanInput {
            namespace: self.config.namespace.clone(), instance_id: id.to_string(), implementation,
            image: self.config.server_image.clone(), service_type: self.config.service_type.clone(),
            storage_class: self.config.storage_class.clone(), storage_size: self.config.storage_size.clone(),
            server_port, cpu_request: self.config.cpu_request.clone(), memory_request: self.config.memory_request.clone(),
            command: (!command.is_empty()).then(|| command.to_string()), args: args.to_vec(), env: env.clone(),
            working_dir: work_dir.to_str().map(ToString::to_string), labels: BTreeMap::new(),
            annotations: BTreeMap::new(), readiness_path: Some(self.config.readiness_path.clone()),
        };
        self.command(&["apply", "-f", "-"], Some(&kubernetes::object_list(&input).to_string()), deadline)?;
        self.status(id)?.ok_or_else(|| "kubernetes objects applied but observation is absent".to_string())
    }

    fn stop(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String> {
        self.require_access()?;
        let name = format!("deployment/lkjmc-{id}");
        self.command(&["scale", &name, "--replicas=0"], None, deadline)?;
        let selector = kubernetes::selector(id);
        self.command(&["wait", "--for=delete", "pod", "-l", &selector, &format!("--timeout={}s", deadline.as_secs())], None, deadline)?;
        Ok(RuntimeObservation::absent("kubernetes workload observed at zero pods"))
    }

    fn status(&self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        let selector = kubernetes::selector(id);
        let output = self.command(&["get", "pods", "-l", &selector, "-o", "json"], None, Duration::from_secs(3))?;
        let Some(value) = kubernetes::observe_pods_json(&output)? else { return Ok(None) };
        Ok(Some(if value.ready {
            RuntimeObservation { observed_state: "kubernetes-ready".into(), healthy: true, identity: None,
                message: Some(format!("ready pod observed; restarts={}", value.restart_count)) }
        } else { RuntimeObservation::unhealthy(value.last_error.or(value.phase).unwrap_or_else(|| "pod not ready".into())) }))
    }

    fn logs(&self, id: &str, _root: &str, lines: usize) -> Result<Vec<String>, String> {
        let selector = kubernetes::selector(id);
        let output = self.command(&["logs", "-l", &selector, "--tail", &lines.to_string()], None, Duration::from_secs(3))?;
        Ok(output.lines().map(ToString::to_string).collect())
    }

    fn delete(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String> {
        self.require_access()?;
        let selector = kubernetes::selector(id);
        self.command(&["delete", "deployment,service,pvc", "-l", &selector, "--ignore-not-found=true"], None, deadline)?;
        Ok(RuntimeObservation::absent("kubernetes owned objects deleted"))
    }

    fn shutdown(&self, _deadline: Duration) -> Result<(), String> { Ok(()) }
}
