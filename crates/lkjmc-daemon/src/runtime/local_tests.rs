use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static NEXT_TEMP_ROOT: AtomicUsize = AtomicUsize::new(0);

use super::{LocalRuntime, StopFault};
use crate::runtime::process;

#[test]
fn early_exit_not_success() -> Result<(), String> {
    let root = temp_root("lkjmc-early-exit")?;
    let observation = LocalRuntime::new().start(
        "early-exit",
        "/bin/false",
        &[],
        &BTreeMap::new(),
        root.to_str().ok_or("temporary path is not UTF-8")?,
        &root,
        Duration::from_secs(1),
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
    let runtime = LocalRuntime::new();
    let mut identity = process::identity(child.id())?;
    identity.start_ticks = identity.start_ticks.saturating_add(1);
    assert!(!runtime.recover("fenced", identity).healthy);
    assert!(runtime
        .start(
            "fenced",
            "/bin/true",
            &[],
            &BTreeMap::new(),
            "/tmp",
            &root,
            Duration::from_secs(1),
        )
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
    let root = temp_root("lkjmc-stop-fault")?;
    let runtime = LocalRuntime::new();
    let pid = start_term_ignoring_group(&runtime, &root)?;
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
    runtime: &LocalRuntime,
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
        Duration::from_secs(1),
    )?;
    observation
        .pid()
        .ok_or_else(|| "missing spawned pid".to_string())
}

#[test]
fn stop_fault_fixtures_are_isolated_in_parallel() -> Result<(), String> {
    let roots = std::thread::scope(|scope| {
        let workers = (0..8)
            .map(|_| scope.spawn(|| temp_root("lkjmc-stop-fault")))
            .collect::<Vec<_>>();
        let mut roots = Vec::new();
        for worker in workers {
            roots.push(worker.join().map_err(|_| "fixture worker panicked")??);
        }
        Ok::<_, String>(roots)
    })?;
    let distinct_count = roots
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let root_count = roots.len();
    for root in roots {
        let _ = std::fs::remove_dir_all(root);
    }
    assert_eq!(distinct_count, root_count);
    Ok(())
}

fn temp_root(prefix: &str) -> Result<std::path::PathBuf, String> {
    for _ in 0..100 {
        let sequence = NEXT_TEMP_ROOT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{sequence}", std::process::id()));
        match std::fs::create_dir(&root) {
            Ok(()) => return Ok(root),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("create temporary root: {error}")),
        }
    }
    Err("create unique temporary root: exhausted attempts".to_string())
}
