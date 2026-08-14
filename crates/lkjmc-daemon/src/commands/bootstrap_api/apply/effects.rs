mod instances;
pub(super) mod readiness;
mod secrets;

use lkjmc_core::command::CommandEnvelope;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::network_plan::NetworkEffect;
use crate::app::AppState;
use crate::support::instance_helpers::store;

pub fn apply_effect(
    state: &AppState,
    _request: &CommandEnvelope,
    effect: &NetworkEffect,
) -> Result<(), String> {
    match effect {
        NetworkEffect::EnsureRoots => ensure_roots(state),
        NetworkEffect::GenerateForwardingSecret { path } => secrets::ensure_secret_file(path),
        NetworkEffect::ReconcileInstance { id, shape } => {
            secrets::read_secret(&shape.forwarding_secret_file)?;
            let mut client = state.database_connection()?;
            instances::reconcile(
                &mut client,
                id.as_str(),
                instances::InstanceShape {
                    kind: shape.kind,
                    server_port: shape.server_port,
                    memory_mb: shape.memory_mb,
                    bind_host: &shape.bind_host,
                    public_hosts: &shape.public_hosts,
                    backend_addresses: &shape.backend_addresses,
                    forwarding_secret_file: &shape.forwarding_secret_file,
                    online_mode: shape.online_mode,
                    daemon_http_url: &shape.daemon_http_url,
                    _daemon_http_token_file: &shape.daemon_http_token_file,
                    eula_accepted: shape.eula_accepted,
                    server_asset_path: &shape.server_asset_path,
                    server_asset_sha256: &shape.server_asset_sha256,
                },
            )
        }
        NetworkEffect::RenderInstance { id } => render(state, id.as_str()),
        NetworkEffect::StartInstance { .. } | NetworkEffect::StopInstance { .. } => {
            Err("network runtime effect must use the fenced runtime adapter".to_string())
        }
        NetworkEffect::WaitForReadiness { .. } => {
            Err("network readiness must run without a database connection".to_string())
        }
    }
}

fn ensure_roots(state: &AppState) -> Result<(), String> {
    let roots = [
        state.config_root(),
        state.data_root(),
        state.log_root(),
        state.jar_root(),
        state.asset_root(),
    ];
    for root in roots {
        ensure_dir(&root)?;
    }
    let socket_path = state.socket_path();
    let parent = Path::new(&socket_path)
        .parent()
        .ok_or_else(|| format!("socket path has no parent: {socket_path}"))?;
    ensure_dir(parent.to_string_lossy().as_ref())
}

fn ensure_dir(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("root path must not be empty".to_string());
    }
    let existed = Path::new(path).exists();
    std::fs::create_dir_all(path).map_err(|error| format!("create {path}: {error}"))?;
    if !Path::new(path).is_dir() {
        return Err(format!("root is not a directory: {path}"));
    }
    #[cfg(unix)]
    {
        if !existed {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o750))
                .map_err(|error| format!("chmod {path}: {error}"))?;
        }
        let mode = std::fs::metadata(path)
            .map_err(|error| format!("stat {path}: {error}"))?
            .permissions()
            .mode();
        if mode & 0o022 != 0 {
            return Err(format!("root is group/other writable: {path}"));
        }
    }
    Ok(())
}

fn render(state: &AppState, id: &str) -> Result<(), String> {
    render_with(state, id, |kind, config| {
        crate::templates::render_instance(state, id, kind, config).map(|_| ())
    })
}

pub(super) fn render_with(
    state: &AppState,
    id: &str,
    renderer: impl FnOnce(&str, &serde_json::Value) -> Result<(), String>,
) -> Result<(), String> {
    let (instance, config) = {
        let mut client = state.database_connection()?;
        let instance = store(lkjmc_store::instance::get(&mut client, id))?
            .ok_or_else(|| format!("instance not found: {id}"))?;
        let config = store(lkjmc_store::instance::config(&mut client, id))?
            .ok_or_else(|| format!("instance config not found: {id}"))?;
        (instance, config)
    };
    renderer(&instance.kind, &config)
}

pub fn apply_runtime_effect(state: &AppState, effect: &NetworkEffect) -> Result<(), String> {
    let (id, running) = match effect {
        NetworkEffect::StartInstance { id } => (id.as_str(), true),
        NetworkEffect::StopInstance { id } => (id.as_str(), false),
        _ => return Err("network effect is not a runtime effect".to_string()),
    };
    if !running {
        return crate::support::instance_helpers::stop_runtime(state, id).map(|_| ());
    }
    {
        let mut client = state.database_connection()?;
        store(lkjmc_store::instance::update_desired_state(
            &mut client,
            id,
            "running",
        ))?;
    }
    match crate::support::instance_helpers::start_runtime(state, id) {
        Ok(observation) if observation.healthy => Ok(()),
        Ok(observation) => Err(observation
            .message
            .unwrap_or_else(|| format!("instance failed to start: {id}"))),
        Err(error) => {
            let mut client = state.database_connection()?;
            store(lkjmc_store::instance::update_desired_state(
                &mut client,
                id,
                "failed",
            ))?;
            Err(error)
        }
    }
}

#[cfg(test)]
mod root_tests {
    use super::ensure_dir;

    #[test]
    fn existing_safe_root_does_not_require_metadata_write() -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!(
            "lkjmc-existing-root-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir(&root).map_err(|error| error.to_string())?;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o710))
            .map_err(|error| error.to_string())?;
        ensure_dir(root.to_string_lossy().as_ref())?;
        let mode = std::fs::metadata(&root)
            .map_err(|error| error.to_string())?
            .permissions()
            .mode()
            & 0o777;
        std::fs::remove_dir(&root).map_err(|error| error.to_string())?;
        assert_eq!(mode, 0o710);
        Ok(())
    }

    #[test]
    fn writable_existing_root_is_rejected() -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!(
            "lkjmc-writable-root-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir(&root).map_err(|error| error.to_string())?;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o770))
            .map_err(|error| error.to_string())?;
        let result = ensure_dir(root.to_string_lossy().as_ref());
        std::fs::remove_dir(&root).map_err(|error| error.to_string())?;
        assert!(result.is_err());
        Ok(())
    }
}
