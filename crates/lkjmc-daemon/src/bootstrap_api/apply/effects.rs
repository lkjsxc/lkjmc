mod secrets;

use lkjmc_core::bootstrap::{BootstrapEffect, ServerProject};
use lkjmc_core::command::CommandEnvelope;
use lkjmc_core::id::CommandId;
use lkjmc_core::instance::InstanceKind;
use serde_json::{json, Value};
use std::time::Duration;
use uuid::Uuid;

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
        BootstrapEffect::ReconcileInstance {
            id,
            kind,
            server_port,
            memory_mb,
        } => reconcile(client, id.as_str(), *kind, *server_port, *memory_mb),
        BootstrapEffect::RenderInstance { id } => render(state, client, id.as_str()),
        BootstrapEffect::InstallPlugin { id, plugin } => {
            crate::plugin_install::install(state, client, id.as_str(), *plugin).map(|_| ())
        }
        BootstrapEffect::StartInstance { id } => start(state, client, id.as_str()),
        BootstrapEffect::RestartInstance { id } => {
            let _ = crate::instance_helpers::stop_runtime(state, client, id.as_str());
            start(state, client, id.as_str())
        }
        BootstrapEffect::WaitForReadiness { id } => wait_running(state, id.as_str()),
        _ => Ok(()),
    }
}

fn reconcile(
    client: &mut postgres::Client,
    id: &str,
    kind: InstanceKind,
    server_port: u16,
    memory_mb: u32,
) -> Result<(), String> {
    let jar = store(lkjmc_store::jar::latest_matching(client, project(kind)))?
        .ok_or_else(|| format!("server jar asset not found for {}", kind_text(kind)))?;
    let config = instance_config(id, kind, server_port, memory_mb, jar.id)?;
    if lkjmc_store::instance::get(client, id)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        store(lkjmc_store::instance::update_config(client, id, &config))?;
    } else {
        store(lkjmc_store::instance::insert(
            client,
            id,
            None,
            kind_text(kind),
            "stopped",
            &config,
        ))?;
    }
    let _ = lkjmc_store::instance::reserve_port(client, id, i32::from(server_port), "server");
    store(lkjmc_store::instance::set_jar_asset(client, id, jar.id))?;
    Ok(())
}

fn instance_config(
    id: &str,
    kind: InstanceKind,
    server_port: u16,
    memory_mb: u32,
    jar_id: Uuid,
) -> Result<Value, String> {
    let secret = secrets::read_secret("/etc/lkjmc/forwarding.secret")?;
    let mut config = json!({
        "template": if kind == InstanceKind::Velocity {"velocity-modern"} else {"paper-survival"},
        "serverPort": server_port,
        "memoryMb": memory_mb,
        "jarAssetId": jar_id.to_string(),
        "forwardingSecret": secret,
        "proxyOnlineMode": true,
        "env": {
            "LKJMC_INSTANCE_ID": id,
            "LKJMC_DAEMON_HTTP_URL": "http://127.0.0.1:8765",
            "LKJMC_DAEMON_HTTP_TOKEN_FILE": "/etc/lkjmc/daemon-http.token"
        }
    });
    if id == "hub" {
        config["eulaAccepted"] = json!(true);
        config["velocityProxy"] = json!(true);
        config["properties"] = json!({"motd":"lkjmc hub", "gamemode":"survival"});
    }
    if id == "proxy" {
        config["hubAddress"] = json!("127.0.0.1:25566");
    }
    Ok(config)
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
    crate::instance_helpers::start_runtime(state, client, id).map(|_| ())
}

fn wait_running(state: &AppState, id: &str) -> Result<(), String> {
    for _ in 0..30 {
        if crate::instance_helpers::runtime_running(state, id)? {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(format!("instance did not become ready: {id}"))
}

fn project(kind: InstanceKind) -> &'static str {
    match kind {
        InstanceKind::Velocity => "velocity",
        InstanceKind::Paper => "paper",
        InstanceKind::Folia => "folia",
        InstanceKind::VanillaCustom | InstanceKind::ModdedCustom => "paper",
    }
}

fn kind_text(kind: InstanceKind) -> &'static str {
    match kind {
        InstanceKind::Velocity => "velocity",
        InstanceKind::Paper => "paper",
        InstanceKind::Folia => "folia",
        InstanceKind::VanillaCustom => "vanilla-custom",
        InstanceKind::ModdedCustom => "modded-custom",
    }
}

fn project_text(project: ServerProject) -> &'static str {
    match project {
        ServerProject::Paper => "paper",
        ServerProject::Folia => "folia",
        ServerProject::Velocity => "velocity",
    }
}
