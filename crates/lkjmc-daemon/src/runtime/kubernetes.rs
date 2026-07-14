use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use lkjmc_core::config::KubernetesRuntimeConfig;
use lkjmc_core::kubernetes;

use crate::runtime::adapter::{require, RuntimeRequirements};

#[path = "kubernetes_command.rs"]
mod command;
use crate::runtime::{RuntimeAdapter, RuntimeCapabilities, RuntimeObservation};

pub struct KubernetesRuntime {
    config: KubernetesRuntimeConfig,
    kubectl_program: PathBuf,
}

impl KubernetesRuntime {
    pub fn new(config: KubernetesRuntimeConfig) -> Self {
        Self {
            config,
            kubectl_program: PathBuf::from("kubectl"),
        }
    }

    #[cfg(test)]
    pub(super) fn with_kubectl_program(config: KubernetesRuntimeConfig, program: PathBuf) -> Self {
        Self {
            config,
            kubectl_program: program,
        }
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
            storage: false,
            secrets: false,
            configuration: false,
            logs: true,
            recovery: false,
        }
    }

    fn check_capabilities(&self) -> Result<(), String> {
        require(
            self.capabilities(),
            RuntimeRequirements {
                readiness: true,
                logs: true,
                ..RuntimeRequirements::default()
            },
        )?;
        self.require_access(Duration::from_secs(3))
    }

    fn runtime_start(
        &self,
        _id: &str,
        _command: &str,
        _args: &[String],
        _env: &BTreeMap<String, String>,
        _log_root: &str,
        _work_dir: &Path,
        _deadline: Duration,
    ) -> Result<RuntimeObservation, String> {
        Err("kubernetes start unsupported: launch files, configuration, and secrets are not mounted".to_string())
    }

    fn runtime_stop(&self, _id: &str, _deadline: Duration) -> Result<RuntimeObservation, String> {
        Err("kubernetes stop unsupported: durable operation/fence ownership and resourceVersion preconditions are unavailable".to_string())
    }

    fn runtime_status(&self, id: &str) -> Result<Option<RuntimeObservation>, String> {
        let selector = kubernetes::selector(id);
        let deadline = command::CommandDeadline::new(Duration::from_secs(3));
        let output = self.command(
            &["get", "pods", "-l", &selector, "-o", "json"],
            None,
            &deadline,
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

    fn runtime_adopt(
        &self,
        _id: &str,
        _identity: crate::runtime::ProcessIdentity,
    ) -> Result<RuntimeObservation, String> {
        Err("kubernetes process identity adoption unsupported".to_string())
    }

    fn runtime_logs(&self, id: &str, _root: &str, lines: usize) -> Result<Vec<String>, String> {
        let selector = kubernetes::selector(id);
        let deadline = command::CommandDeadline::new(Duration::from_secs(3));
        let output = self.command(
            &["logs", "-l", &selector, "--tail", &lines.to_string()],
            None,
            &deadline,
        )?;
        Ok(output.lines().map(ToString::to_string).collect())
    }

    fn runtime_delete(&self, _id: &str, _deadline: Duration) -> Result<RuntimeObservation, String> {
        Err("kubernetes delete unsupported: durable operation/fence ownership and atomic UID preconditions are unavailable".to_string())
    }

    fn runtime_shutdown(&self, _deadline: Duration) -> Result<(), String> {
        Ok(())
    }
}
