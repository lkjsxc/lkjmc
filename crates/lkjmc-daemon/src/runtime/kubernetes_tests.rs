use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use lkjmc_core::config::KubernetesRuntimeConfig;

use super::kubernetes::KubernetesRuntime;
use super::RuntimeAdapter;

fn denied_runtime() -> KubernetesRuntime {
    KubernetesRuntime::new(KubernetesRuntimeConfig {
        namespace: format!("lkjmc-denied-{}", std::process::id()),
        kubeconfig_path: Some("/definitely/missing/lkjmc-kubeconfig".to_string()),
        in_cluster: false,
        server_image: "example.invalid/minecraft@sha256:deadbeef".to_string(),
        service_type: "ClusterIP".to_string(),
        storage_class: "denied".to_string(),
        storage_size: "1Gi".to_string(),
        log_tail_lines: 20,
        readiness_path: "/ready".to_string(),
        cpu_request: "100m".to_string(),
        memory_request: "1Gi".to_string(),
    })
}

#[test]
fn kubernetes_plan_fails_closed_without_access() {
    let runtime = denied_runtime();
    assert!(runtime.check_capabilities().is_err());
    let result = runtime.start(
        "denied",
        "java",
        &[],
        &BTreeMap::new(),
        "/tmp",
        Path::new("/tmp"),
        Duration::from_millis(200),
    );
    assert!(result.is_err());
}
