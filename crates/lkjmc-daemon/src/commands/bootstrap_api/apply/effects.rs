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
        NetworkEffect::ReconcileInstance {
            id,
            kind,
            server_port,
            memory_mb,
            bind_host,
            public_hosts,
            backend_address,
            forwarding_secret_file,
            online_mode,
            daemon_http_url,
            daemon_http_token_file,
        } => {
            secrets::read_secret(forwarding_secret_file)?;
            let mut client = state.database_connection()?;
            instances::reconcile(
                &mut client,
                id.as_str(),
                instances::InstanceShape {
                    kind: *kind,
                    server_port: *server_port,
                    memory_mb: *memory_mb,
                    bind_host,
                    public_hosts,
                    backend_address: backend_address.as_deref(),
                    forwarding_secret_file,
                    online_mode: *online_mode,
                    daemon_http_url,
                    _daemon_http_token_file: daemon_http_token_file,
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
    std::fs::create_dir_all(path).map_err(|error| format!("create {path}: {error}"))?;
    if !Path::new(path).is_dir() {
        return Err(format!("root is not a directory: {path}"));
    }
    #[cfg(unix)]
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o750))
        .map_err(|error| format!("chmod {path}: {error}"))?;
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
