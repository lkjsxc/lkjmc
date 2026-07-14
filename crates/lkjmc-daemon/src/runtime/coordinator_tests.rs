use super::LifecycleCoordinator;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn unrelated_key_proceeds_while_key_is_held() -> Result<(), String> {
    let coordinator = LifecycleCoordinator::new();
    let held_id = format!("held-{}", uuid::Uuid::new_v4().simple());
    let peer_id = format!("peer-{}", uuid::Uuid::new_v4().simple());
    let held = coordinator.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let holder = std::thread::spawn(move || {
        held.run(&held_id, || {
            use std::os::unix::process::CommandExt;
            let mut command = std::process::Command::new("sleep");
            command.arg("5").process_group(0);
            let mut child = command.spawn().map_err(|error| error.to_string())?;
            entered_tx.send(()).map_err(|error| error.to_string())?;
            let released = release_rx.recv().map_err(|error| error.to_string());
            let _ = child.kill();
            let _ = child.wait();
            released
        })
    });
    entered_rx.recv().map_err(|error| error.to_string())?;
    let (peer_tx, peer_rx) = mpsc::channel();
    let peer = coordinator.clone();
    std::thread::spawn(move || {
        peer_tx.send(peer.run(&peer_id, || {
            let status = std::process::Command::new("/bin/true")
                .status()
                .map_err(|error| error.to_string())?;
            status
                .success()
                .then_some(())
                .ok_or("peer child failed".to_string())
        }))
    });
    peer_rx
        .recv_timeout(Duration::from_millis(200))
        .map_err(|_| "unrelated instance blocked".to_string())??;
    release_tx.send(()).map_err(|error| error.to_string())?;
    holder.join().map_err(|_| "holder panicked".to_string())??;
    Ok(())
}

#[test]
fn same_instance_race_is_serialized() -> Result<(), String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    let coordinator = LifecycleCoordinator::new();
    let instance_id = Arc::new(format!("same-{}", uuid::Uuid::new_v4().simple()));
    let active = Arc::new(AtomicUsize::new(0));
    let maximum = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(std::sync::Barrier::new(17));
    let (done_tx, done_rx) = mpsc::channel();
    let mut workers = Vec::new();
    for _ in 0..16 {
        let coordinator = coordinator.clone();
        let instance_id = Arc::clone(&instance_id);
        let active = Arc::clone(&active);
        let maximum = Arc::clone(&maximum);
        let barrier = Arc::clone(&barrier);
        let done = done_tx.clone();
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            let result = coordinator.run(&instance_id, || {
                let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                maximum.fetch_max(current, Ordering::SeqCst);
                let status = std::process::Command::new("/bin/true")
                    .status()
                    .map_err(|error| error.to_string())?;
                std::thread::sleep(Duration::from_millis(2));
                active.fetch_sub(1, Ordering::SeqCst);
                status
                    .success()
                    .then_some(())
                    .ok_or("race child failed".into())
            });
            let _ = done.send(result);
        }));
    }
    barrier.wait();
    for _ in 0..16 {
        done_rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|_| "same-instance race timed out".to_string())??;
    }
    for worker in workers {
        worker.join().map_err(|_| "worker panicked".to_string())?;
    }
    assert_eq!(maximum.load(Ordering::SeqCst), 1);
    Ok(())
}

#[test]
fn runtime_load_budget() -> Result<(), String> {
    let coordinator = LifecycleCoordinator::new();
    let started = std::time::Instant::now();
    let mut workers = Vec::new();
    let run_id = uuid::Uuid::new_v4();
    for index in 0..64 {
        let coordinator = coordinator.clone();
        workers.push(std::thread::spawn(move || {
            coordinator.run(&format!("instance-{run_id}-{index}"), || Ok(()))
        }));
    }
    for worker in workers {
        worker
            .join()
            .map_err(|_| "load worker panicked".to_string())??;
    }
    if started.elapsed() > Duration::from_secs(2) {
        return Err("runtime load budget exceeded".to_string());
    }
    assert_eq!(coordinator.key_count()?, 0);
    Ok(())
}
