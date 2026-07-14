use std::collections::BTreeMap;
use std::time::Duration;

use super::{LocalRuntime, StopFault};
use crate::runtime::process;
use crate::runtime::test_support::{temp_root, unique_id};

struct ChildGuard(std::process::Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn early_exit_not_success() -> Result<(), String> {
    let root = temp_root("lkjmc-early-exit")?;
    let runtime = LocalRuntime::with_data_root(&root);
    let observation = runtime.runtime_start(
        &unique_id("early-exit"),
        "/bin/false",
        &[],
        &BTreeMap::new(),
        path(&root)?,
        &root,
        Duration::from_secs(1),
    )?;
    std::fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    assert!(!observation.healthy);
    Ok(())
}

#[test]
fn pid_recovery_fenced() -> Result<(), String> {
    let root = temp_root("lkjmc-fenced")?;
    let mut child = sleep_group()?;
    let runtime = LocalRuntime::with_data_root(&root);
    let id = unique_id("fenced");
    let mut identity = process::identity(child.0.id())?;
    identity.start_ticks = identity.start_ticks.saturating_add(1);
    assert!(!runtime.recover(&id, identity).healthy);
    assert!(runtime
        .runtime_start(
            &id,
            "/bin/true",
            &[],
            &BTreeMap::new(),
            path(&root)?,
            &root,
            Duration::from_secs(1),
        )
        .is_err());
    child.0.kill().map_err(|error| error.to_string())?;
    child.0.wait().map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn pid_start_and_executable_mismatches_are_fenced() -> Result<(), String> {
    let child = sleep_group()?;
    let identity = process::identity(child.0.id())?;
    let mut wrong_pid = identity.clone();
    wrong_pid.pid = u32::MAX;
    assert!(
        !LocalRuntime::new()
            .recover(&unique_id("wrong-pid"), wrong_pid)
            .healthy
    );
    let mut wrong_start = identity.clone();
    wrong_start.start_ticks = wrong_start.start_ticks.saturating_add(1);
    assert!(
        !LocalRuntime::new()
            .recover(&unique_id("wrong-start"), wrong_start)
            .healthy
    );
    let mut wrong_executable = identity;
    wrong_executable.executable_inode = wrong_executable.executable_inode.saturating_add(1);
    assert!(
        !LocalRuntime::new()
            .recover(&unique_id("wrong-executable"), wrong_executable)
            .healthy
    );
    Ok(())
}

#[test]
fn shutdown_respects_total_deadline() -> Result<(), String> {
    let root = temp_root("lkjmc-bounded-shutdown")?;
    let runtime = LocalRuntime::with_data_root(&root);
    let args = vec!["5".to_string()];
    let observation = runtime.runtime_start(
        &unique_id("bounded"),
        "sleep",
        &args,
        &BTreeMap::new(),
        path(&root)?,
        &root,
        Duration::from_secs(1),
    )?;
    let pid = observation
        .pid()
        .ok_or("shutdown process identity missing")?;
    let started = std::time::Instant::now();
    runtime.runtime_shutdown(Duration::from_millis(200))?;
    let elapsed = started.elapsed();
    std::fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    assert!(
        elapsed < Duration::from_secs(2),
        "shutdown took {elapsed:?}"
    );
    assert!(!process::group_exists(pid));
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
    let root = temp_root("lkjmc-stop-fault")?;
    let runtime = LocalRuntime::with_data_root(&root);
    let id = unique_id("faulted");
    let pid = start_term_ignoring_group(&runtime, &root, &id)?;
    let result = (|| {
        runtime.inject_stop_fault(fault);
        assert!(runtime
            .runtime_stop(&id, Duration::from_millis(20))
            .is_err());
        assert!(process::group_exists(pid));
        assert!(runtime
            .runtime_status(&id)?
            .is_some_and(|value| value.healthy));
        assert!(
            !runtime
                .runtime_stop(&id, Duration::from_millis(500))?
                .healthy
        );
        assert!(!process::group_exists(pid));
        assert!(runtime.runtime_status(&id)?.is_none());
        Ok(())
    })();
    let cleanup = runtime.runtime_shutdown(Duration::from_secs(1));
    if cleanup.is_err() {
        let _ = process::kill_group(pid);
        let _ = runtime.runtime_shutdown(Duration::from_secs(1));
    }
    let _ = std::fs::remove_dir_all(root);
    result.and(cleanup)
}

fn start_term_ignoring_group(
    runtime: &LocalRuntime,
    root: &std::path::Path,
    id: &str,
) -> Result<u32, String> {
    let args = vec![
        "-c".to_string(),
        "trap '' TERM; while :; do sleep 1; done".to_string(),
    ];
    runtime
        .runtime_start(
            id,
            "sh",
            &args,
            &BTreeMap::new(),
            path(root)?,
            root,
            Duration::from_secs(1),
        )?
        .pid()
        .ok_or_else(|| "missing spawned pid".to_string())
}

fn sleep_group() -> Result<ChildGuard, String> {
    use std::os::unix::process::CommandExt;
    let mut command = std::process::Command::new("sleep");
    command.arg("5").process_group(0);
    command
        .spawn()
        .map(ChildGuard)
        .map_err(|error| error.to_string())
}

fn path(root: &std::path::Path) -> Result<&str, String> {
    root.to_str().ok_or("temporary path is not UTF-8".into())
}
