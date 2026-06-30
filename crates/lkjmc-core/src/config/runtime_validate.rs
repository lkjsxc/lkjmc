use crate::config::{RuntimeAdapter, RuntimeConfig};
use crate::error::ConfigError;

use super::validate::{require_kebab, require_non_empty, require_path, require_positive};

pub(super) fn validate_kubernetes_runtime(runtime: &RuntimeConfig) -> Result<(), ConfigError> {
    if runtime.adapter != RuntimeAdapter::Kubernetes {
        return Ok(());
    }
    let Some(kubernetes) = &runtime.kubernetes else {
        return Err(ConfigError::invalid("runtime.kubernetes", "is required"));
    };
    require_kebab("runtime.kubernetes.namespace", &kubernetes.namespace)?;
    if let Some(path) = &kubernetes.kubeconfig_path {
        require_path("runtime.kubernetes.kubeconfigPath", path)?;
    } else if !kubernetes.in_cluster {
        return Err(ConfigError::invalid(
            "runtime.kubernetes.kubeconfigPath",
            "or inCluster is required",
        ));
    }
    require_non_empty("runtime.kubernetes.serverImage", &kubernetes.server_image)?;
    require_non_empty("runtime.kubernetes.serviceType", &kubernetes.service_type)?;
    require_non_empty("runtime.kubernetes.storageClass", &kubernetes.storage_class)?;
    require_non_empty("runtime.kubernetes.storageSize", &kubernetes.storage_size)?;
    require_positive("runtime.kubernetes.logTailLines", kubernetes.log_tail_lines)?;
    require_non_empty(
        "runtime.kubernetes.readinessPath",
        &kubernetes.readiness_path,
    )?;
    require_non_empty("runtime.kubernetes.cpuRequest", &kubernetes.cpu_request)?;
    require_non_empty(
        "runtime.kubernetes.memoryRequest",
        &kubernetes.memory_request,
    )
}
