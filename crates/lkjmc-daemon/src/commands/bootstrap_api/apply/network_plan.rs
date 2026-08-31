use std::collections::BTreeMap;
use std::net::{IpAddr, SocketAddr};

use lkjmc_core::config::{AssetKind, LkjmcConfig};
use lkjmc_core::id::InstanceId;
use lkjmc_core::instance::InstanceKind;
use lkjmc_core::network_intent::{ChangeAction, NetworkInspection};
use uuid::Uuid;

use crate::app::AppState;

pub(super) struct ReconcileShape {
    pub kind: InstanceKind,
    pub server_port: u16,
    pub memory_mb: u32,
    pub bind_host: String,
    pub public_hosts: Vec<String>,
    pub backend_addresses: BTreeMap<String, String>,
    pub default_backend: Option<String>,
    pub forwarding_secret_file: String,
    pub online_mode: bool,
    pub daemon_http_url: String,
    pub server_asset_path: String,
    pub server_asset_sha256: String,
}

pub(super) enum NetworkEffect {
    EnsureRoots,
    GenerateForwardingSecret {
        path: String,
    },
    ReconcileInstance {
        id: InstanceId,
        shape: Box<ReconcileShape>,
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
    let server_asset = server_asset(config, &instance.asset_ids)?;
    effects.push(NetworkEffect::ReconcileInstance {
        id: parse_id(id)?,
        shape: Box::new(ReconcileShape {
            kind: instance.kind,
            server_port: listener.port,
            memory_mb: instance.memory_mb,
            bind_host: listener.bind_host.clone(),
            public_hosts: listener.public_hosts.clone(),
            backend_addresses: backend_addresses(config)?,
            default_backend: default_backend(config),
            forwarding_secret_file: config.network.forwarding.secret_file.clone(),
            online_mode: config.network.auth.online_mode,
            daemon_http_url: http_url(&config.daemon_http.address),
            server_asset_path: server_asset.path.clone(),
            server_asset_sha256: server_asset.sha256.clone(),
        }),
    });
    effects.push(NetworkEffect::RenderInstance { id: parse_id(id)? });
    Ok(())
}

fn default_backend(config: &LkjmcConfig) -> Option<String> {
    config
        .network
        .routes
        .first()
        .map(|route| route.target.clone())
}

fn server_asset<'a>(
    config: &'a LkjmcConfig,
    asset_ids: &[String],
) -> Result<&'a lkjmc_core::config::NetworkAsset, String> {
    let matches = asset_ids
        .iter()
        .filter_map(|id| config.network.assets.iter().find(|asset| asset.id == *id))
        .filter(|asset| asset.kind == AssetKind::Server)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [asset] => Ok(*asset),
        [] => Err("network instance has no server asset".to_string()),
        _ => Err("network instance has multiple server assets".to_string()),
    }
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

fn backend_addresses(config: &LkjmcConfig) -> Result<BTreeMap<String, String>, String> {
    let mut addresses = BTreeMap::new();
    for instance in config
        .network
        .instances
        .iter()
        .filter(|instance| instance.kind != InstanceKind::Velocity)
    {
        let listener = config
            .network
            .listener(&instance.listener)
            .ok_or_else(|| format!("backend listener missing: {}", instance.id))?;
        addresses.insert(
            instance.id.clone(),
            listener_socket(&listener.bind_host, listener.port)?,
        );
    }
    if addresses.is_empty() {
        return Err("network has no backend instances".to_string());
    }
    Ok(addresses)
}

fn listener_socket(host: &str, port: u16) -> Result<String, String> {
    let address = host
        .parse::<IpAddr>()
        .map_err(|_| "listener host is not a literal IP address".to_string())?;
    Ok(SocketAddr::new(address, port).to_string())
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

#[cfg(test)]
mod tests {
    use super::listener_socket;

    #[test]
    fn listener_socket_formats_ipv6_without_ambiguity() -> Result<(), String> {
        assert_eq!(listener_socket("::1", 25566)?, "[::1]:25566");
        assert!(listener_socket("localhost", 25566).is_err());
        Ok(())
    }
}
