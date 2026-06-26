mod instances;
mod readiness;
mod secrets;

use instances::InstanceShape;
use lkjmc_core::bootstrap::{BootstrapEffect, ServerProject};
use lkjmc_core::command::CommandEnvelope;
use lkjmc_core::id::CommandId;
use serde_json::json;

use crate::app::AppState;
use crate::instance_helpers::store;

pub fn apply_effect(
    state: &AppState,
    request: &CommandEnvelope,
    client: &mut postgres::Client,
    effect: &BootstrapEffect,
) -> Result<(), String> {
    match effect {
        BootstrapEffect::GenerateDaemonHttpToken => {
            secrets::ensure_secret_file("/etc/lkjmc/daemon-http.token")
        }
        BootstrapEffect::GenerateForwardingSecret => {
            secrets::ensure_secret_file("/etc/lkjmc/forwarding.secret")
        }
        BootstrapEffect::SyncServerAsset { project } => sync_server(state, request, *project),
        BootstrapEffect::RegisterLocalPlugin { plugin } => {
            crate::plugin_assets::register_local(state, client, *plugin).map(|_| ())
        }
        BootstrapEffect::SyncPluginAsset { plugin } => {
            crate::plugin_downloads::sync(state, client, *plugin).map(|_| ())
        }
        BootstrapEffect::ReconcileInstance {
            id,
            kind,
            server_port,
            memory_mb,
            bind_host,
            public_hosts,
        } => instances::reconcile(
            client,
            id.as_str(),
            InstanceShape {
                kind: *kind,
                server_port: *server_port,
                memory_mb: *memory_mb,
                bind_host,
                public_hosts,
            },
        ),
        BootstrapEffect::RenderInstance { id } => render(state, client, id.as_str()),
        BootstrapEffect::InstallPlugin { id, plugin } => {
            crate::plugin_install::install(state, client, id.as_str(), *plugin).map(|_| ())
        }
        BootstrapEffect::StartInstance { id } => start(state, client, id.as_str()),
        BootstrapEffect::RestartInstance { id } => {
            let _ = crate::instance_helpers::stop_runtime(state, client, id.as_str());
            start(state, client, id.as_str())
        }
        BootstrapEffect::WaitForReadiness { id } => {
            readiness::wait_running(state, client, id.as_str())
        }
        _ => Ok(()),
    }
}

fn sync_server(
    state: &AppState,
    request: &CommandEnvelope,
    project: ServerProject,
) -> Result<(), String> {
    let response = crate::downloads::handle(
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
    let observation = crate::instance_helpers::start_runtime(state, client, id)?;
    if observation.healthy {
        Ok(())
    } else {
        Err(observation
            .message
            .unwrap_or_else(|| format!("instance failed to start: {id}")))
    }
}

fn project_text(project: ServerProject) -> &'static str {
    match project {
        ServerProject::Paper => "paper",
        ServerProject::Folia => "folia",
        ServerProject::Velocity => "velocity",
    }
}
