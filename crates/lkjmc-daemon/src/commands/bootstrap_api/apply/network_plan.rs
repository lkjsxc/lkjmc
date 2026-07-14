use lkjmc_core::config::{AssetKind, LkjmcConfig};
use lkjmc_core::id::InstanceId;
use lkjmc_core::instance::InstanceKind;
use lkjmc_core::network_intent::{ChangeAction, NetworkInspection};
use uuid::Uuid;

use crate::app::AppState;

pub(super) enum NetworkEffect {
    EnsureRoots,
    GenerateForwardingSecret {
        path: String,
    },
    ReconcileInstance {
        id: InstanceId,
        kind: InstanceKind,
        server_port: u16,
        memory_mb: u32,
        bind_host: String,
        public_hosts: Vec<String>,
        backend_address: Option<String>,
        forwarding_secret_file: String,
        online_mode: bool,
        daemon_http_url: String,
        daemon_http_token_file: String,
    },
    RenderInstance {
        id: InstanceId,
    },
    StartInstance {
        id: InstanceId,
    },
    StopInstance {
        id: InstanceId,
    },
    WaitForReadiness {
        id: InstanceId,
    },
}

pub(super) fn effects(
    config: &LkjmcConfig,
    inspection: &NetworkInspection,
) -> Result<Vec<NetworkEffect>, String> {
    let mut effects = vec![NetworkEffect::EnsureRoots];
    for change in &inspection.changes {
        match change.action {
            ChangeAction::VerifyAsset => {}
            ChangeAction::EnsureSecret => effects.push(NetworkEffect::GenerateForwardingSecret {
                path: config.network.forwarding.secret_file.clone(),
            }),
            ChangeAction::Render => {
                render_effect(config, change.instance_id.as_deref(), &mut effects)?
            }
            ChangeAction::Start => effects.push(NetworkEffect::StartInstance {
                id: parse_change_id(change.instance_id.as_deref())?,
            }),
            ChangeAction::Stop => effects.push(NetworkEffect::StopInstance {
                id: parse_change_id(change.instance_id.as_deref())?,
            }),
            ChangeAction::VerifyReadiness => effects.push(NetworkEffect::WaitForReadiness {
                id: parse_change_id(change.instance_id.as_deref())?,
            }),
        }
    }
    Ok(effects)
}

fn render_effect(
    config: &LkjmcConfig,
    id: Option<&str>,
    effects: &mut Vec<NetworkEffect>,
) -> Result<(), String> {
    let id = id.ok_or("network render change has no instance")?;
    let instance = config
        .network
        .instances
        .iter()
        .find(|item| item.id == id)
        .ok_or("network render instance is absent")?;
    let listener = config
        .network
        .listener(&instance.listener)
        .ok_or("network render listener is absent")?;
    effects.push(NetworkEffect::ReconcileInstance {
        id: parse_id(id)?,
        kind: instance.kind,
        server_port: listener.port,
        memory_mb: instance.memory_mb,
        bind_host: listener.bind_host.clone(),
        public_hosts: listener.public_hosts.clone(),
        backend_address: backend_address(config, id),
        forwarding_secret_file: config.network.forwarding.secret_file.clone(),
        online_mode: config.network.auth.online_mode,
        daemon_http_url: http_url(&config.daemon_http.address),
        daemon_http_token_file: config.daemon_http.token_file.clone(),
    });
    effects.push(NetworkEffect::RenderInstance { id: parse_id(id)? });
    Ok(())
}

pub(super) fn register_assets(state: &AppState, config: &LkjmcConfig) -> Result<(), String> {
    let assets = config
        .network
        .assets
        .iter()
        .filter(|asset| asset.kind == AssetKind::Server)
        .map(|asset| {
            let size = std::fs::metadata(&asset.path)
                .map_err(|error| error.to_string())?
                .len();
            let project = config
                .network
                .instances
                .iter()
                .find(|instance| instance.asset_ids.iter().any(|id| id == &asset.id))
                .map(|instance| project(instance.kind))
                .ok_or_else(|| format!("server asset has no instance: {}", asset.id))?;
            Ok((asset, project, size))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut database = state.database_connection()?;
    for (asset, project, size) in assets {
        if let Some(existing) = lkjmc_store::jar::get_by_path(&mut database, &asset.path)
            .map_err(|error| error.to_string())?
        {
            if !existing.sha256.eq_ignore_ascii_case(&asset.sha256) {
                return Err(format!("registered asset digest differs: {}", asset.id));
            }
            continue;
        }
        lkjmc_store::jar::insert(
            &mut database,
            lkjmc_store::jar::NewJarAsset {
                id: Uuid::new_v4(),
                kind: project,
                project,
                channel: "immutable",
                name: &asset.id,
                path: &asset.path,
                sha256: &asset.sha256,
                size_bytes: i64::try_from(size).map_err(|_| "asset size exceeds bigint")?,
                source: "network-intent",
            },
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn backend_address(config: &LkjmcConfig, id: &str) -> Option<String> {
    let route = config.network.routes.iter().find(|route| {
        config
            .network
            .listener(&route.listener)
            .is_some_and(|listener| {
                listener.id
                    == config
                        .network
                        .instances
                        .iter()
                        .find(|instance| instance.id == id)
                        .map(|instance| instance.listener.as_str())
                        .unwrap_or_default()
            })
    })?;
    let target = config
        .network
        .instances
        .iter()
        .find(|item| item.id == route.target)?;
    let listener = config.network.listener(&target.listener)?;
    Some(format!("{}:{}", listener.bind_host, listener.port))
}
fn parse_change_id(id: Option<&str>) -> Result<InstanceId, String> {
    parse_id(id.ok_or("network instance change has no instance")?)
}
fn parse_id(id: &str) -> Result<InstanceId, String> {
    InstanceId::parse(id.to_string()).map_err(|error| error.to_string())
}
fn http_url(address: &str) -> String {
    if address.starts_with("http://") || address.starts_with("https://") {
        address.to_string()
    } else {
        format!("http://{address}")
    }
}
fn project(kind: InstanceKind) -> &'static str {
    match kind {
        InstanceKind::Velocity => "velocity",
        InstanceKind::Folia => "folia",
        InstanceKind::Purpur => "purpur",
        _ => "paper",
    }
}
