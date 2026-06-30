use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeConfig {
    pub adapter: RuntimeAdapter,
    pub default_java_memory_mb: u32,
    #[serde(default = "super::defaults::proxy_java_memory_mb")]
    pub proxy_java_memory_mb: u32,
    pub stop_timeout_seconds: u32,
    #[serde(default = "super::defaults::port_range_start")]
    pub port_range_start: u16,
    #[serde(default = "super::defaults::port_range_end")]
    pub port_range_end: u16,
    #[serde(default)]
    pub kubernetes: Option<KubernetesRuntimeConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeAdapter {
    LocalProcess,
    Kubernetes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KubernetesRuntimeConfig {
    pub namespace: String,
    #[serde(default)]
    pub kubeconfig_path: Option<String>,
    #[serde(default)]
    pub in_cluster: bool,
    pub server_image: String,
    pub service_type: String,
    pub storage_class: String,
    pub storage_size: String,
    pub log_tail_lines: u32,
    pub readiness_path: String,
    pub cpu_request: String,
    pub memory_request: String,
}
