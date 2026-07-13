use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use lkjmc_core::config::KubernetesRuntimeConfig;
use lkjmc_core::kubernetes::{self, KubernetesPlanInput};

use crate::runtime::adapter::{require, RuntimeRequirements};

#[path = "kubernetes_command.rs"]
mod command;
use crate::runtime::{RuntimeAdapter, RuntimeCapabilities, RuntimeObservation};

pub struct KubernetesRuntime {
    config: KubernetesRuntimeConfig,
}

impl KubernetesRuntime {
    pub fn new(config: KubernetesRuntimeConfig) -> Self {
        Self { config }
    }
}

impl RuntimeAdapter for KubernetesRuntime {
    fn name(&self) -> &'static str {
        "kubernetes"
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            process_identity: false,
            readiness: true,
            storage: true,
            secrets: false,
            configuration: false,
            logs: true,
            recovery: true,
        }
    }

    fn check_capabilities(&self) -> Result<(), String> {
        require(
            self.capabilities(),
            RuntimeRequirements {
                readiness: true,
                storage: true,
                logs: true,
                recovery: true,
                ..RuntimeRequirements::default()
            },
        )?;
        self.require_access()
    }

    fn start(
        &self,
        id: &str,
        command: &str,
        args: &[String],
        env: &BTreeMap<String, String>,
        _log_root: &str,
        work_dir: &Path,
        deadline: Duration,
    ) -> Result<RuntimeObservation, String> {
        self.require_access()?;
        let server_port = env
            .get("LKJMC_SERVER_PORT")
            .and_then(|value| value.parse().ok())
            .ok_or("kubernetes launch requires proved server port")?;
        let implementation = env
            .get("LKJMC_INSTANCE_KIND")
            .cloned()
            .ok_or("kubernetes launch requires implementation kind")?;
        let input = KubernetesPlanInput {
            namespace: self.config.namespace.clone(),
            instance_id: id.to_string(),
            implementation,
            image: self.config.server_image.clone(),
            service_type: self.config.service_type.clone(),
            storage_class: self.config.storage_class.clone(),
            storage_size: self.config.storage_size.clone(),
            server_port,
            cpu_request: self.config.cpu_request.clone(),
            memory_request: self.config.memory_request.clone(),
            command: (!command.is_empty()).then(|| command.to_string()),
            args: args.to_vec(),
            env: env.clone(),
            working_dir: work_dir.to_str().map(ToString::to_string),
            labels: BTreeMap::new(),
            annotations: BTreeMap::new(),
            readiness_path: Some(self.config.readiness_path.clone()),
        };
        self.command(
            &["apply", "-f", "-"],
            Some(&kubernetes::object_list(&input).to_string()),
            deadline,
        )?;
        self.status(id)?
            .ok_or_else(|| "kubernetes objects applied but observation is absent".to_string())
    }

    fn stop(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String> {
        self.require_access()?;
        let name = format!("deployment/lkjmc-{id}");
        self.command(&["scale", &name, "--replicas=0"], None, deadline)?;
        let selector = kubernetes::selector(id);
        self.command(
            &[
                "wait",
                "--for=delete",
                "pod",
                "-l",
                &selector,
                &format!("--timeout={}s", deadline.as_secs()),
            ],
            None,
            deadline,
        )?;
        Ok(RuntimeObservation::absent(
            "kubernetes workload observed at zero pods",
        ))
    }

    fn status(&self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        let selector = kubernetes::selector(id);
        let output = self.command(
            &["get", "pods", "-l", &selector, "-o", "json"],
            None,
            Duration::from_secs(3),
        )?;
        let Some(value) = kubernetes::observe_pods_json(&output)? else {
            return Ok(None);
        };
        Ok(Some(if value.ready {
            RuntimeObservation {
                observed_state: "kubernetes-ready".into(),
                healthy: true,
                identity: None,
                message: Some(format!(
                    "ready pod observed; restarts={}",
                    value.restart_count
                )),
            }
        } else {
            RuntimeObservation::unhealthy(
                value
                    .last_error
                    .or(value.phase)
                    .unwrap_or_else(|| "pod not ready".into()),
            )
        }))
    }

    fn adopt(
        &self,
        _id: &str,
        _identity: crate::runtime::ProcessIdentity,
    ) -> Result<RuntimeObservation, String> {
        Err("kubernetes process identity adoption unsupported".to_string())
    }

    fn logs(&self, id: &str, _root: &str, lines: usize) -> Result<Vec<String>, String> {
        let selector = kubernetes::selector(id);
        let output = self.command(
            &["logs", "-l", &selector, "--tail", &lines.to_string()],
            None,
            Duration::from_secs(3),
        )?;
        Ok(output.lines().map(ToString::to_string).collect())
    }

    fn delete(&self, id: &str, deadline: Duration) -> Result<RuntimeObservation, String> {
        self.require_access()?;
        let selector = kubernetes::selector(id);
        self.command(
            &[
                "delete",
                "deployment,service,pvc",
                "-l",
                &selector,
                "--ignore-not-found=true",
            ],
            None,
            deadline,
        )?;
        Ok(RuntimeObservation::absent(
            "kubernetes owned objects deleted",
        ))
    }

    fn shutdown(&self, _deadline: Duration) -> Result<(), String> {
        Ok(())
    }
}
