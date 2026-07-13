use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

#[derive(Clone, Default)]
pub struct LifecycleCoordinator {
    state: Arc<State>,
}

#[derive(Default)]
struct State {
    keys: Mutex<BTreeMap<String, Weak<Mutex<()>>>>,
    accepting: Mutex<bool>,
}

impl LifecycleCoordinator {
    pub fn new() -> Self {
        let value = Self::default();
        if let Ok(mut accepting) = value.state.accepting.lock() {
            *accepting = true;
        }
        value
    }

    pub fn run<T>(&self, id: &str, work: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        if !self.accepting()? {
            return Err("runtime is shutting down".to_string());
        }
        let guard = self.guard(id)?;
        let _instance = guard
            .lock()
            .map_err(|_| "instance lifecycle lock poisoned".to_string())?;
        if !self.accepting()? {
            return Err("runtime is shutting down".to_string());
        }
        work()
    }

    pub fn close(&self) {
        if let Ok(mut accepting) = self.state.accepting.lock() {
            *accepting = false;
        }
    }

    fn accepting(&self) -> Result<bool, String> {
        self.state
            .accepting
            .lock()
            .map(|value| *value)
            .map_err(|_| "runtime admission lock poisoned".to_string())
    }

    fn guard(&self, id: &str) -> Result<Arc<Mutex<()>>, String> {
        let mut keys = self
            .state
            .keys
            .lock()
            .map_err(|_| "lifecycle key map poisoned".to_string())?;
        keys.retain(|_, value| value.strong_count() > 0);
        if let Some(guard) = keys.get(id).and_then(Weak::upgrade) {
            return Ok(guard);
        }
        let guard = Arc::new(Mutex::new(()));
        keys.insert(id.to_string(), Arc::downgrade(&guard));
        Ok(guard)
    }
}

#[cfg(test)]
mod tests {
    use super::LifecycleCoordinator;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn unrelated_key_proceeds_while_key_is_held() -> Result<(), String> {
        let coordinator = LifecycleCoordinator::new();
        let held = coordinator.clone();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let holder = std::thread::spawn(move || {
            held.run("held", || {
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
            peer_tx.send(peer.run("peer", || {
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
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let mut workers = Vec::new();
        for _ in 0..16 {
            let coordinator = coordinator.clone();
            let active = Arc::clone(&active);
            let maximum = Arc::clone(&maximum);
            workers.push(std::thread::spawn(move || {
                coordinator.run("same", || {
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    maximum.fetch_max(current, Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(2));
                    active.fetch_sub(1, Ordering::SeqCst);
                    Ok(())
                })
            }));
        }
        for worker in workers {
            worker.join().map_err(|_| "worker panicked".to_string())??;
        }
        assert_eq!(maximum.load(Ordering::SeqCst), 1);
        Ok(())
    }

    #[test]
    fn runtime_load_budget() -> Result<(), String> {
        let coordinator = LifecycleCoordinator::new();
        let started = std::time::Instant::now();
        let mut workers = Vec::new();
        for index in 0..64 {
            let coordinator = coordinator.clone();
            workers.push(std::thread::spawn(move || {
                coordinator.run(&format!("instance-{index}"), || Ok(()))
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
        Ok(())
    }
}
