use std::sync::{Arc, Mutex};

use crate::runtime_local::LocalRuntime;

#[derive(Clone)]
pub struct AppState {
    pub database_url: Option<String>,
    pub runtime: Arc<Mutex<LocalRuntime>>,
    pub log_root: String,
    pub jar_root: String,
    pub data_root: String,
}

impl AppState {
    pub fn with_roots(
        database_url: Option<String>,
        log_root: String,
        jar_root: String,
        data_root: String,
    ) -> Self {
        Self {
            database_url,
            runtime: Arc::new(Mutex::new(LocalRuntime::new())),
            log_root,
            jar_root,
            data_root,
        }
    }
}
