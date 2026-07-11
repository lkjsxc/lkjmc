use std::collections::BTreeMap;
use std::time::Duration;

use super::{LocalRuntime, StopFault};
use crate::runtime::process;

#[test]
fn early_exit_not_success() -> Result<(), String> {
    let root = temp_root("lkjmc-early-exit");
    let observation = LocalRuntime::new().start(
        "early-exit",
        "/bin/false",
        &[],
        &BTreeMap::new(),
        root.to_str().ok_or("temporary path is not UTF-8")?,
        &root,
    )?;
    std::fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    assert!(!observation.healthy);
    Ok(())
}

#[test]
fn pid_recovery_fenced() -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    let root = std::env::temp_dir();
    let mut command = std::process::Command::new("sleep");
    command.arg("5").process_group(0);
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let mut runtime = LocalRuntime::new();
    assert!(!runtime.recover("fenced", child.id()).healthy);
    assert!(runtime
        .start("fenced", "/bin/true", &[], &BTreeMap::new(), "/tmp", &root)
        .is_err());
    child.kill().map_err(|error| error.to_string())?;
    child.wait().map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn stop_signal_failure_retry_does_not_report_live_group_absent() -> Result<(), String> {
    stop_fault_retry_keeps_actual_group_tracked(StopFault::Signal)
}

#[test]
fn stop_wait_failure_retry_does_not_report_live_group_absent() -> Result<(), String> {
    stop_fault_retry_keeps_actual_group_tracked(StopFault::Wait)
}

fn stop_fault_retry_keeps_actual_group_tracked(fault: StopFault) -> Result<(), String> {
    let root = temp_root("lkjmc-stop-fault");
    let mut runtime = LocalRuntime::new();
    let pid = start_term_ignoring_group(&mut runtime, &root)?;
    let result = (|| {
        runtime.inject_stop_fault(fault);
        assert!(runtime.stop("faulted", Duration::from_millis(20)).is_err());
        assert!(process::group_exists(pid));
        assert!(runtime
            .status("faulted")?
            .is_some_and(|value| value.healthy));
        assert!(!runtime.stop("faulted", Duration::from_millis(20))?.healthy);
        assert!(!process::group_exists(pid));
        assert!(runtime.status("faulted")?.is_none());
        Ok(())
    })();
    let _ = process::kill_group(pid);
    let _ = std::fs::remove_dir_all(root);
    result
}

fn start_term_ignoring_group(
    runtime: &mut LocalRuntime,
    root: &std::path::Path,
) -> Result<u32, String> {
    let args = vec![
        "-c".to_string(),
        "trap '' TERM; while :; do sleep 1; done".to_string(),
    ];
    let observation = runtime.start(
        "faulted",
        "sh",
        &args,
        &BTreeMap::new(),
        root.to_str().ok_or("temporary path is not UTF-8")?,
        root,
    )?;
    observation
        .pid
        .ok_or_else(|| "missing spawned pid".to_string())
}

fn temp_root(prefix: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("{prefix}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::create_dir_all(&root);
    root
}
