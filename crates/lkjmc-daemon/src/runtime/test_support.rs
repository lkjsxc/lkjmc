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

pub(crate) fn materialize_test_eula(
    data_root: &std::path::Path,
    instance_id: &str,
) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let id = lkjmc_core::id::InstanceId::parse(instance_id.to_string())
        .map_err(|error| error.to_string())?;
    let directory = data_root.join(id.as_str());
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("create test instance root: {error}"))?;
    let path = directory.join("eula.txt");
    std::fs::write(&path, b"eula=true\n").map_err(|error| format!("write test EULA: {error}"))?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640))
        .map_err(|error| format!("set test EULA mode: {error}"))
}
