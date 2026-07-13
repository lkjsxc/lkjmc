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
        let instance = guard
            .lock()
            .map_err(|_| "instance lifecycle lock poisoned".to_string())?;
        let result = if self.accepting()? {
            work()
        } else {
            Err("runtime is shutting down".to_string())
        };
        drop(instance);
        self.cleanup(id, &guard)?;
        result
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

    fn cleanup(&self, id: &str, guard: &Arc<Mutex<()>>) -> Result<(), String> {
        let mut keys = self
            .state
            .keys
            .lock()
            .map_err(|_| "lifecycle key map poisoned".to_string())?;
        if Arc::strong_count(guard) == 1
            && keys
                .get(id)
                .and_then(Weak::upgrade)
                .is_some_and(|value| Arc::ptr_eq(&value, guard))
        {
            keys.remove(id);
        }
        Ok(())
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

    #[cfg(test)]
    fn key_count(&self) -> Result<usize, String> {
        self.state
            .keys
            .lock()
            .map(|keys| keys.len())
            .map_err(|_| "lifecycle key map poisoned".to_string())
    }
}

#[cfg(test)]
#[path = "coordinator_tests.rs"]
mod tests;
