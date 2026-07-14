use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use lkjmc_core::config::KubernetesRuntimeConfig;

use super::kubernetes::KubernetesRuntime;
use super::RuntimeAdapter;

fn denied_config() -> KubernetesRuntimeConfig {
    KubernetesRuntimeConfig {
        namespace: format!("lkjmc-denied-{}", uuid::Uuid::new_v4().simple()),
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
    }
}

fn denied_runtime() -> KubernetesRuntime {
    KubernetesRuntime::new(denied_config())
}

fn fake_kubectl(body: &str) -> Result<PathBuf, String> {
    let path =
        std::env::temp_dir().join(format!("lkjmc-kubectl-{}", uuid::Uuid::new_v4().simple()));
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).map_err(|error| error.to_string())?;
    let mut permissions = fs::metadata(&path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&path, permissions).map_err(|error| error.to_string())?;
    Ok(path)
}

fn rejected<T>(result: Result<T, String>) -> Result<String, String> {
    match result {
        Err(error) => Ok(error),
        Ok(_) => Err("Kubernetes operation unexpectedly succeeded".to_string()),
    }
}

#[test]
fn kubernetes_plan_fails_closed_without_access() -> Result<(), String> {
    let runtime = denied_runtime();
    assert!(runtime.check_capabilities().is_err());
    let started = Instant::now();
    let result = runtime.runtime_start(
        "denied",
        "java",
        &[],
        &BTreeMap::new(),
        "/tmp",
        Path::new("/host/rendered/instance"),
        Duration::from_millis(200),
    );
    assert!(rejected(result)?.contains("not mounted"));
    assert!(started.elapsed() < Duration::from_millis(200));
    Ok(())
}

#[test]
fn kubernetes_hung_kubectl_respects_total_deadline() -> Result<(), String> {
    let program = fake_kubectl("exec sleep 10")?;
    let runtime = KubernetesRuntime::with_kubectl_program(denied_config(), program.clone());
    let started = Instant::now();
    let error = rejected(runtime.require_access(Duration::from_millis(200)))?;
    let elapsed = started.elapsed();
    let _ = fs::remove_file(program);
    assert!(error.contains("deadline elapsed"));
    assert!(elapsed >= Duration::from_millis(150), "elapsed={elapsed:?}");
    assert!(elapsed < Duration::from_millis(750), "elapsed={elapsed:?}");
    Ok(())
}

#[test]
fn kubernetes_destructive_paths_deny_before_effect() -> Result<(), String> {
    let marker = std::env::temp_dir().join(format!(
        "lkjmc-kubectl-effect-{}",
        uuid::Uuid::new_v4().simple()
    ));
    let _ = fs::remove_file(&marker);
    let program = fake_kubectl(&format!("touch {}", marker.display()))?;
    let runtime = KubernetesRuntime::with_kubectl_program(denied_config(), program.clone());

    let stop = rejected(runtime.runtime_stop("wrong-instance", Duration::from_millis(200)))?;
    let delete = rejected(runtime.runtime_delete("wrong-instance", Duration::from_millis(200)))?;

    let _ = fs::remove_file(program);
    assert!(stop.contains("operation/fence ownership"));
    assert!(stop.contains("resourceVersion"));
    assert!(delete.contains("operation/fence ownership"));
    assert!(delete.contains("UID preconditions"));
    let marker_exists = marker.exists();
    let _ = fs::remove_file(marker);
    assert!(!marker_exists, "destructive kubectl effect was invoked");
    Ok(())
}
