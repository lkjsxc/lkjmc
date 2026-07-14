use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustix::fs::{mkfifoat, Mode, CWD};
use uuid::Uuid;

use crate::app::AppState;

#[test]
fn bundle_rejects_nested_parent_and_target_symlinks_without_outside_writes() -> Result<(), String> {
    let root = unique_root("symlinks");
    let outside = unique_root("outside");
    fs::create_dir_all(root.join("approved")).map_err(|error| error.to_string())?;
    fs::create_dir_all(&outside).map_err(|error| error.to_string())?;
    symlink(&outside, root.join("approved/nested")).map_err(|error| error.to_string())?;
    let nested_output = root.join("approved/nested/support.tar");
    assert!(crate::support::bundle::create(&state(None, &root), &nested_output).is_err());
    assert!(!outside.join("support.tar").exists());

    let target = root.join("approved/support.tar");
    let outside_target = outside.join("target");
    fs::write(&outside_target, b"outside sentinel").map_err(|error| error.to_string())?;
    symlink(&outside_target, &target).map_err(|error| error.to_string())?;
    assert!(crate::support::bundle::create(&state(None, &root), &target).is_err());
    assert_eq!(
        fs::read(&outside_target).map_err(|error| error.to_string())?,
        b"outside sentinel"
    );
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    fs::remove_dir_all(outside).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn nested_parent_symlink_swaps_never_write_outside() -> Result<(), String> {
    let Some(database) = database()? else {
        return Ok(());
    };
    let root = unique_root("swap");
    let outside = unique_root("swap-outside");
    let parent = root.join("approved");
    let live = parent.join("live");
    let held = parent.join("held");
    fs::create_dir_all(&live).map_err(|error| error.to_string())?;
    fs::create_dir_all(&outside).map_err(|error| error.to_string())?;
    prepare_roots(&root)?;
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = stop.clone();
    let worker_live = live.clone();
    let worker_held = held.clone();
    let worker_outside = outside.clone();
    let worker = std::thread::spawn(move || {
        while !worker_stop.load(Ordering::Acquire) {
            if fs::rename(&worker_live, &worker_held).is_ok() {
                let _ = symlink(&worker_outside, &worker_live);
                let _ = fs::remove_file(&worker_live);
                let _ = fs::rename(&worker_held, &worker_live);
            }
        }
    });
    let output = live.join("support.tar");
    let _ = run_bundle(&state(Some(database.url()), &root), output, None);
    stop.store(true, Ordering::Release);
    worker
        .join()
        .map_err(|_| "symlink swap worker failed".to_string())?;
    assert!(!outside.join("support.tar").exists());
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
    Ok(())
}

#[test]
fn fifo_and_slow_fault_return_bounded_without_partial_output() -> Result<(), String> {
    let Some(database) = database()? else {
        return Ok(());
    };
    let root = unique_root("faults");
    prepare_roots(&root)?;
    let fifo = root.join("logs/daemon.log");
    mkfifoat(CWD, &fifo, Mode::RUSR | Mode::WUSR).map_err(|error| error.to_string())?;
    let fifo_output = root.join("fifo-support.tar");
    let started = Instant::now();
    assert!(run_bundle(
        &state(Some(database.url()), &root),
        fifo_output.clone(),
        Some((Duration::from_secs(2), Duration::ZERO)),
    )
    .is_err());
    assert!(started.elapsed() < Duration::from_millis(2250));
    assert!(!fifo_output.exists());

    fs::remove_file(fifo).map_err(|error| error.to_string())?;
    let slow_output = root.join("slow-support.tar");
    let started = Instant::now();
    let result = run_bundle(
        &state(Some(database.url()), &root),
        slow_output.clone(),
        Some((Duration::from_secs(1), Duration::from_secs(5))),
    );
    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_millis(1250));
    assert!(!slow_output.exists());
    assert!(!fs::read_dir(&root)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .any(|entry| entry
            .file_name()
            .to_string_lossy()
            .starts_with(".lkjmc-support-")));
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

fn run_bundle(
    state: &AppState,
    output: PathBuf,
    fault: Option<(Duration, Duration)>,
) -> Result<serde_json::Value, String> {
    let admission = state.admit_request().ok_or("admission unavailable")?;
    let bundle_state = state.clone();
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    runtime
        .block_on(admission.run_blocking(move || match fault {
            Some((cap, delay)) => {
                crate::support::bundle::create_with_fault(&bundle_state, &output, cap, delay)
            }
            None => crate::support::bundle::create(&bundle_state, &output),
        }))
        .map_err(|error| match error {
            crate::app::BlockingError::Deadline => "outer bundle deadline".to_string(),
            crate::app::BlockingError::Join => "bundle worker failed".to_string(),
        })?
}

fn prepare_roots(root: &Path) -> Result<(), String> {
    for name in ["logs", "data"] {
        fs::create_dir_all(root.join(name)).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn database() -> Result<Option<crate::test_database::TestDatabase>, String> {
    let Ok(url) = std::env::var("LKJMC_STORE_TEST_DATABASE_URL") else {
        return Ok(None);
    };
    crate::test_database::migrate(&url).map(Some)
}

fn unique_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("lkjmc-obs-{label}-{}", Uuid::new_v4().simple()))
}

fn state(database_url: Option<&str>, root: &Path) -> AppState {
    AppState::with_config_path(
        database_url.map(str::to_string),
        8,
        root.join("config").to_string_lossy().to_string(),
        root.join("logs").to_string_lossy().to_string(),
        root.join("jars").to_string_lossy().to_string(),
        root.join("data").to_string_lossy().to_string(),
        None,
        None,
        None,
    )
}
