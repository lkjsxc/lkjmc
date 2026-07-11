mod instances;
mod readiness;
mod secrets;

use instances::InstanceShape;
use lkjmc_core::bootstrap::{BootstrapEffect, ServerProject};
use lkjmc_core::command::CommandEnvelope;
use lkjmc_core::id::CommandId;
use serde_json::json;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::app::AppState;
use crate::support::instance_helpers::store;

pub fn apply_effect(
    state: &AppState,
    request: &CommandEnvelope,
    client: &mut postgres::Client,
    effect: &BootstrapEffect,
) -> Result<(), String> {
    match effect {
        BootstrapEffect::EnsureRoots => ensure_roots(state),
        BootstrapEffect::EnsureMigrations => ensure_migrations(client),
        BootstrapEffect::GenerateDaemonHttpToken { path } => secrets::ensure_secret_file(path),
        BootstrapEffect::GenerateForwardingSecret { path } => secrets::ensure_secret_file(path),
        BootstrapEffect::SyncServerAsset { project } => sync_server(state, request, *project),
        BootstrapEffect::RegisterLocalPlugin { plugin } => {
            crate::assets::plugin_assets::register_local(state, client, *plugin).map(|_| ())
        }
        BootstrapEffect::SyncPluginAsset { plugin } => {
            crate::assets::plugin_downloads::sync(state, client, *plugin).map(|_| ())
        }
        BootstrapEffect::ReconcileInstance {
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
        } => instances::reconcile(
            client,
            id.as_str(),
            InstanceShape {
                kind: *kind,
                server_port: *server_port,
                memory_mb: *memory_mb,
                bind_host,
                public_hosts,
                backend_address: backend_address.as_deref(),
                forwarding_secret_file,
                online_mode: *online_mode,
                daemon_http_url,
                daemon_http_token_file,
            },
        ),
        BootstrapEffect::RenderInstance { id } => render(state, client, id.as_str()),
        BootstrapEffect::InstallPlugin { id, plugin } => {
            crate::assets::plugin_install::install(state, client, id.as_str(), *plugin).map(|_| ())
        }
        BootstrapEffect::StartInstance { id } => start(state, client, id.as_str()),
        BootstrapEffect::RestartInstance { id } => {
            crate::support::instance_helpers::stop_runtime(state, client, id.as_str())?;
            start(state, client, id.as_str())
        }
        BootstrapEffect::WaitForReadiness { id } => {
            readiness::wait_running(state, client, id.as_str())
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

fn ensure_migrations(client: &mut postgres::Client) -> Result<(), String> {
    lkjmc_store::migrate::apply(client)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn sync_server(
    state: &AppState,
    request: &CommandEnvelope,
    project: ServerProject,
) -> Result<(), String> {
    let response = crate::commands::downloads::handle(
        state,
        CommandEnvelope {
            request_id: CommandId::internal("bootstrap-server-sync"),
            actor: request.actor.clone(),
            command: "jar.sync".to_string(),
            body: json!({"project": project_text(project), "channel": "stable"}),
        },
    );
    if response.ok {
        Ok(())
    } else {
        Err(response
            .error
            .map(|error| error.message)
            .unwrap_or_else(|| "server asset sync failed".to_string()))
    }
}

fn render(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
    let instance = store(lkjmc_store::instance::get(client, id))?
        .ok_or_else(|| format!("instance not found: {id}"))?;
    let config = store(lkjmc_store::instance::config(client, id))?
        .ok_or_else(|| format!("instance config not found: {id}"))?;
    crate::templates::render_instance(state, id, &instance.kind, &config).map(|_| ())
}

fn start(state: &AppState, client: &mut postgres::Client, id: &str) -> Result<(), String> {
    store(lkjmc_store::instance::update_desired_state(
        client, id, "running",
    ))?;
    match crate::support::instance_helpers::start_runtime(state, client, id) {
        Ok(observation) if observation.healthy => Ok(()),
        Ok(observation) => Err(observation
            .message
            .unwrap_or_else(|| format!("instance failed to start: {id}"))),
        Err(error) => {
            store(lkjmc_store::instance::update_desired_state(
                client, id, "failed",
            ))?;
            Err(error)
        }
    }
}

fn project_text(project: ServerProject) -> &'static str {
    match project {
        ServerProject::Paper => "paper",
        ServerProject::Folia => "folia",
        ServerProject::Purpur => "purpur",
        ServerProject::Velocity => "velocity",
    }
}
