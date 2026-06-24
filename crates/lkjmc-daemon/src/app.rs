use std::sync::{Arc, Mutex};

use crate::runtime_local::LocalRuntime;

#[derive(Clone)]
pub struct AppState {
    pub database_url: Option<String>,
    pub runtime: Arc<Mutex<LocalRuntime>>,
    pub log_root: String,
}

impl AppState {
    pub fn with_roots(database_url: Option<String>, log_root: String) -> Self {
        Self {
            database_url,
            runtime: Arc::new(Mutex::new(LocalRuntime::new())),
            log_root,
        }
    }
}
