use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::Duration;

use lkjmc_core::config::{AssetKind, NetworkConfig, NetworkInstance};
use lkjmc_core::instance::{DesiredState, InstanceKind};
use lkjmc_core::network_intent::{
    self, InspectionOutcome, NetworkInspection, NetworkObservation, ResourceObservation,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;
use crate::runtime::RuntimeGoal;

pub(super) fn inspect(state: &AppState) -> Result<NetworkInspection, String> {
    let config = state
        .runtime_config()?
        .ok_or("runtime config is unavailable")?;
    let observed = observation(state, &config.network)?;
    let mut inspection = network_intent::inspect(&config.network, &observed);
    if inspection.outcome != InspectionOutcome::Blocked {
        inspection
            .unsupported
            .extend(asset_failures(&config.network));
        if !inspection.unsupported.is_empty() {
            inspection.outcome = InspectionOutcome::Blocked;
            inspection.changes.clear();
        }
    }
    Ok(inspection)
}

fn observation(
    state: &AppState,
    intent: &lkjmc_core::config::NetworkConfig,
) -> Result<NetworkObservation, String> {
    if state.database_url().is_none() {
        return Ok(NetworkObservation::default());
    }
    let (intent_digest, stored) = {
        let mut client = state.database_connection()?;
        let desired = lkjmc_store::network_intent::latest_desired(&mut client)
            .map_err(|error| error.to_string())?;
        let digest = desired
            .filter(|value| value.intent_digest == intent.digest())
            .map(|value| value.intent_digest);
        let mut rows = BTreeMap::new();
        for instance in &intent.instances {
            let row = lkjmc_store::instance::get(&mut client, &instance.id)
                .map_err(|error| error.to_string())?;
            let config = lkjmc_store::instance::config(&mut client, &instance.id)
                .map_err(|error| error.to_string())?;
            let asset_bound = exact_asset_binding(&mut client, intent, instance, config.as_ref())?;
            rows.insert(instance.id.clone(), (row, config, asset_bound));
        }
        (digest, rows)
    };
    let mut resources = BTreeMap::new();
    for instance in &intent.instances {
        let has_shape = stored
            .get(&instance.id)
            .is_some_and(|(row, config, asset_bound)| {
                row.is_some() && config.is_some() && *asset_bound
            });
        let runtime = if has_shape {
            crate::runtime::reconcile::reconcile(
                state,
                &instance.id,
                RuntimeGoal::Observe,
                Uuid::new_v4(),
            )?
        } else {
            state
                .runtime()
                .runtime_status(&instance.id)?
                .unwrap_or_else(|| crate::runtime::RuntimeObservation::absent("runtime is absent"))
        };
        let listener = intent
            .listener(&instance.listener)
            .ok_or_else(|| format!("listener missing for {}", instance.id))?;
        let address = socket_address(&listener.bind_host, listener.port)?;
        let files_ready = rendered_files_ready(
            state,
            &instance.id,
            instance.kind,
            listener.port,
            &listener.bind_host,
        );
        let listener_ready = runtime.healthy
            && TcpStream::connect_timeout(&address, Duration::from_millis(50)).is_ok();
        let blocked = runtime_block(&runtime, address, instance.desired_state);
        resources.insert(
            instance.id.clone(),
            ResourceObservation {
                spec_digest: if files_ready && has_shape {
                    intent.resource_digest(&instance.id)
                } else {
                    "drift".to_string()
                },
                runtime_present: runtime.identity.is_some(),
                ready: runtime.healthy && listener_ready,
                blocked,
            },
        );
    }
    Ok(NetworkObservation {
        intent_digest,
        forwarding_secret_ready: private_file(&intent.forwarding.secret_file),
        resources,
    })
}

fn runtime_block(
    runtime: &crate::runtime::RuntimeObservation,
    address: SocketAddr,
    desired: DesiredState,
) -> Option<String> {
    if !runtime.healthy && !runtime.observed_state.contains("absent") {
        return Some("runtime identity is stale or unowned; apply denied".to_string());
    }
    if desired == DesiredState::Running
        && !runtime.healthy
        && TcpStream::connect_timeout(&address, Duration::from_millis(50)).is_ok()
    {
        return Some(format!(
            "unowned process or listener occupies {address}; apply denied"
        ));
    }
    None
}

fn exact_asset_binding(
    client: &mut postgres::Client,
    intent: &NetworkConfig,
    instance: &NetworkInstance,
    config: Option<&serde_json::Value>,
) -> Result<bool, String> {
    let assets = instance
        .asset_ids
        .iter()
        .filter_map(|id| intent.assets.iter().find(|asset| asset.id == *id))
        .filter(|asset| asset.kind == AssetKind::Server)
        .collect::<Vec<_>>();
    let [asset] = assets.as_slice() else {
        return Ok(false);
    };
    let Some(registered) =
        lkjmc_store::jar::get_by_path(client, &asset.path).map_err(|error| error.to_string())?
    else {
        return Ok(false);
    };
    let configured_id = config
        .and_then(|value| value.get("jarAssetId"))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok());
    let stored_id = lkjmc_store::instance::jar_asset_id(client, &instance.id)
        .map_err(|error| error.to_string())?;
    Ok(registered.sha256.eq_ignore_ascii_case(&asset.sha256)
        && registered.project == asset_project(instance.kind)
        && configured_id == Some(registered.id)
        && stored_id == Some(registered.id))
}

fn asset_project(kind: InstanceKind) -> &'static str {
    match kind {
        InstanceKind::Velocity => "velocity",
        InstanceKind::Folia => "folia",
        InstanceKind::Purpur => "purpur",
        InstanceKind::Paper | InstanceKind::VanillaCustom | InstanceKind::ModdedCustom => "paper",
    }
}

fn rendered_files_ready(
    state: &AppState,
    id: &str,
    kind: InstanceKind,
    port: u16,
    bind_host: &str,
) -> bool {
    let root = Path::new(&state.data_root()).join(id);
    match kind {
        InstanceKind::Velocity => {
            private_text(&root, "velocity.toml").is_some_and(|text| {
                exact_toml_string(&text, "bind", &format!("{bind_host}:{port}"))
            }) && private_text(&root, "forwarding.secret").is_some()
        }
        _ => {
            private_text(&root, "server.properties").is_some_and(|text| {
                exact_property(&text, "server-port", &port.to_string())
                    && exact_property(&text, "server-ip", bind_host)
            }) && private_text(&root, "eula.txt").is_some_and(|text| text.contains("eula=true"))
                && private_text(&root, "spigot.yml").is_some()
                && private_text(&root, "config/paper-global.yml")
                    .is_some_and(|text| text.contains("velocity:"))
        }
    }
}

fn private_text(root: &Path, relative: &str) -> Option<String> {
    let path = root.join(relative);
    if !private_file(path.to_string_lossy().as_ref()) {
        return None;
    }
    std::fs::read_to_string(path).ok()
}

fn exact_property(text: &str, expected_key: &str, expected_value: &str) -> bool {
    let canonical = format!("{expected_key}={expected_value}");
    let mut matched = 0_u8;
    let mut continuation = false;
    for raw_line in text.lines() {
        if continuation {
            continuation = property_line_continues(raw_line);
            continue;
        }
        continuation = property_line_continues(raw_line);
        let line = raw_line.trim_start();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        let Some(key) = decoded_property_key(line) else {
            return false;
        };
        if key == expected_key {
            matched = matched.saturating_add(1);
            if line != canonical {
                return false;
            }
        }
    }
    matched == 1
}

fn decoded_property_key(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut key = String::new();
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'=' || byte == b':' || byte.is_ascii_whitespace() {
            break;
        }
        if byte != b'\\' {
            if !byte.is_ascii() {
                return None;
            }
            key.push(char::from(byte));
            index += 1;
            continue;
        }
        index += 1;
        let escaped = *bytes.get(index)?;
        if escaped == b'u' {
            let digits = bytes.get(index + 1..index + 5)?;
            let digits = std::str::from_utf8(digits).ok()?;
            let value = u32::from_str_radix(digits, 16).ok()?;
            key.push(char::from_u32(value)?);
            index += 5;
            continue;
        }
        key.push(match escaped {
            b't' => '\t',
            b'n' => '\n',
            b'r' => '\r',
            b'f' => '\u{000c}',
            value if value.is_ascii() => char::from(value),
            _ => return None,
        });
        index += 1;
    }
    Some(key)
}

fn property_line_continues(line: &str) -> bool {
    line.as_bytes()
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count()
        % 2
        == 1
}

fn exact_toml_string(text: &str, expected_key: &str, expected_value: &str) -> bool {
    let mut matched = 0_u8;
    let mut top_level = true;
    for line in text.lines() {
        let line = line.trim_start();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            top_level = false;
            continue;
        }
        if !top_level {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key == expected_key
            || key == format!("\"{expected_key}\"")
            || key == format!("'{expected_key}'")
        {
            matched = matched.saturating_add(1);
            if key != expected_key || value.trim() != format!("\"{expected_value}\"") {
                return false;
            }
        }
    }
    matched == 1
}

fn socket_address(host: &str, port: u16) -> Result<SocketAddr, String> {
    format!("{host}:{port}")
        .parse()
        .map_err(|error| format!("invalid listener address: {error}"))
}

fn asset_failures(intent: &lkjmc_core::config::NetworkConfig) -> Vec<String> {
    intent
        .assets
        .iter()
        .filter(|asset| asset.required)
        .filter_map(|asset| {
            if asset.kind == lkjmc_core::config::AssetKind::Plugin {
                return Some(format!(
                    "network capability unsupported: plugin asset adoption: {}",
                    asset.id
                ));
            }
            match file_sha256(&asset.path) {
                Ok(actual) if actual.eq_ignore_ascii_case(&asset.sha256) => None,
                Ok(_) => Some(format!("required asset digest mismatch: {}", asset.id)),
                Err(error) => Some(format!("required asset unavailable: {}: {error}", asset.id)),
            }
        })
        .collect()
}

fn file_sha256(path: &str) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn private_file(path: &str) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.is_file() && metadata.permissions().mode() & 0o077 == 0
    }
    #[cfg(not(unix))]
    metadata.is_file()
}

#[cfg(test)]
mod rendered_file_tests {
    use std::net::{TcpListener, TcpStream};

    use lkjmc_core::instance::DesiredState;

    use super::{exact_property, exact_toml_string, runtime_block};
    use crate::runtime::RuntimeObservation;

    #[test]
    fn property_binding_requires_one_unambiguous_effective_assignment() {
        assert!(exact_property(
            "level-type=minecraft\\:normal\nserver-ip=127.0.0.1\n",
            "server-ip",
            "127.0.0.1"
        ));
        assert!(!exact_property(
            "# server-ip=127.0.0.1\nserver-ip=0.0.0.0\n",
            "server-ip",
            "127.0.0.1"
        ));
        assert!(!exact_property(
            "server-ip=0.0.0.0\nserver-ip=127.0.0.1\n",
            "server-ip",
            "127.0.0.1"
        ));
        assert!(!exact_property(
            "server-ip:127.0.0.1\n",
            "server-ip",
            "127.0.0.1"
        ));
        assert!(!exact_property(
            "server-ip 127.0.0.1\n",
            "server-ip",
            "127.0.0.1"
        ));
        assert!(!exact_property(
            "server\\-ip=0.0.0.0\nserver-ip=127.0.0.1\n",
            "server-ip",
            "127.0.0.1"
        ));
        assert!(!exact_property(
            r#"server\u002dip=0.0.0.0
server-ip=127.0.0.1
"#,
            "server-ip",
            "127.0.0.1"
        ));
        assert!(!exact_property(
            r#"motd=lkjmc\
server-ip=127.0.0.1
"#,
            "server-ip",
            "127.0.0.1"
        ));
    }

    #[test]
    fn active_unowned_listener_is_blocked_but_closed_connection_is_not() -> Result<(), String> {
        let runtime = RuntimeObservation::absent("runtime is absent");
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
        let address = listener.local_addr().map_err(|error| error.to_string())?;
        assert!(runtime_block(&runtime, address, DesiredState::Running).is_some());
        drop(listener);

        let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
        let address = listener.local_addr().map_err(|error| error.to_string())?;
        let client = TcpStream::connect(address).map_err(|error| error.to_string())?;
        let (server, _) = listener.accept().map_err(|error| error.to_string())?;
        drop(server);
        drop(client);
        drop(listener);
        assert!(runtime_block(&runtime, address, DesiredState::Running).is_none());
        Ok(())
    }

    #[test]
    fn velocity_binding_requires_one_plain_top_level_assignment() {
        assert!(exact_toml_string(
            "bind = \"0.0.0.0:25591\"\n[servers]\nhub = \"127.0.0.1:25566\"\n",
            "bind",
            "0.0.0.0:25591"
        ));
        assert!(!exact_toml_string(
            "# bind = \"0.0.0.0:25591\"\nbind = \"127.0.0.1:25591\"\n",
            "bind",
            "0.0.0.0:25591"
        ));
        assert!(!exact_toml_string(
            "bind = \"127.0.0.1:25591\"\nbackend = \"0.0.0.0:25591\"\n",
            "bind",
            "0.0.0.0:25591"
        ));
        assert!(!exact_toml_string(
            "bind = \"0.0.0.0:25591\"\n\"bind\" = \"127.0.0.1:25591\"\n",
            "bind",
            "0.0.0.0:25591"
        ));
    }
}
