use std::sync::Arc;

use crate::app::AppState;

pub(super) struct StateCleanup(pub(super) Arc<AppState>);

impl Drop for StateCleanup {
    fn drop(&mut self) {
        let _ = self.0.shutdown_runtime();
    }
}

pub(super) fn unique_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4().simple())
}

pub(super) fn temp_root(prefix: &str) -> Result<std::path::PathBuf, String> {
    let root = std::env::temp_dir().join(unique_id(prefix));
    std::fs::create_dir(&root).map_err(|error| format!("create temporary root: {error}"))?;
    Ok(root)
}
