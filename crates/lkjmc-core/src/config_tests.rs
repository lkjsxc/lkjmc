use super::*;
use crate::error::ConfigError;
use crate::instance::InstanceKind;

const VALID_MAIN: &str = include_str!("../../../config/defaults/daemon.json.example");
const VALID_INSTANCE: &str = r#"{
  "id":"hub","kind":"paper","desiredState":"running","jarRef":"paper:stable",
  "serverPort":25566,"rconPort":25576,"memoryMb":2048,"template":"paper-survival",
  "properties":{"motd":"lkjmc hub"},"plugins":{"lkjmcPaper":true},
  "sync":{"playerProfile":true,"location":false}
}"#;

fn rejected(input: &str) -> Result<ConfigError, ConfigError> {
    match LkjmcConfig::from_json_str(input) {
        Ok(_) => Err(ConfigError::invalid("test", "expected failure")),
        Err(error) => Ok(error),
    }
}

#[test]
fn valid_main_config_passes() -> Result<(), ConfigError> {
    let config = LkjmcConfig::from_json_str(VALID_MAIN)?;
    assert_eq!(config.install_root, "/opt/lkjmc");
    assert_eq!(config.database.pool_size, 8);
    assert_eq!(config.network.revision, 1);
    assert_eq!(config.network.instances.len(), 2);
    Ok(())
}

#[test]
fn playable_network_helpers_are_derived() -> Result<(), ConfigError> {
    let config = LkjmcConfig::from_json_str(VALID_MAIN)?;
    assert_eq!(
        config.network.java_entry().display_socket(),
        "127.0.0.1:25565"
    );
    assert_eq!(config.network.fallback_server(), "hub");
    assert!(config.network.online_mode());
    assert_eq!(
        config.network.forwarding_secret_file(),
        "/etc/lkjmc/forwarding.secret"
    );
    Ok(())
}

#[test]
fn network_shape_is_closed() -> Result<(), ConfigError> {
    let invalid = VALID_MAIN.replace("\"revision\": 1", "\"revision\": 1, \"compiler\": true");
    assert!(rejected(&invalid)?.to_string().contains("unknown field"));
    Ok(())
}

#[test]
fn network_references_and_bounds_fail_closed() -> Result<(), ConfigError> {
    let dangling = VALID_MAIN.replace("\"listener\": \"hub-java\"", "\"listener\": \"missing\"");
    assert_eq!(
        rejected(&dangling)?.field(),
        Some("network.instances.listener")
    );
    let memory = VALID_MAIN.replace("\"memoryMb\": 2048", "\"memoryMb\": 65537");
    assert_eq!(
        rejected(&memory)?.field(),
        Some("network.instances.memoryMb")
    );
    Ok(())
}

#[test]
fn kubernetes_mount_capabilities_fail_before_use() -> Result<(), ConfigError> {
    let invalid = VALID_MAIN
        .replace("\"adapter\": \"local-process\"", "\"adapter\": \"kubernetes\", \"kubernetes\": {\"namespace\":\"lkjmc-test\",\"kubeconfigPath\":\"/tmp/kube\",\"inCluster\":false,\"serverImage\":\"example.invalid/server@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"serviceType\":\"ClusterIP\",\"storageClass\":\"standard\",\"storageSize\":\"1Gi\",\"logTailLines\":100,\"readinessPath\":\"/ready\",\"cpuRequest\":\"100m\",\"memoryRequest\":\"256Mi\"}")
        .replace("\"runtime\": \"local-process\"", "\"runtime\": \"kubernetes\"")
        .replace("\"mountedSecrets\": true", "\"mountedSecrets\": false");
    let config = LkjmcConfig::from_json_str(&invalid)?;
    let inspection = crate::network_intent::inspect(&config.network, &Default::default());
    assert_eq!(
        inspection.outcome,
        crate::network_intent::InspectionOutcome::Blocked
    );
    assert!(inspection
        .unsupported
        .iter()
        .any(|item| item.contains("mounted-secrets")));
    Ok(())
}

#[test]
fn database_pool_and_paths_are_bounded() -> Result<(), ConfigError> {
    let pool = VALID_MAIN.replace("\"poolSize\": 8", "\"poolSize\": 65");
    assert_eq!(rejected(&pool)?.field(), Some("database.poolSize"));
    let path = VALID_MAIN.replace(
        "\"installRoot\": \"/opt/lkjmc\"",
        "\"installRoot\": \"opt/lkjmc\"",
    );
    assert_eq!(rejected(&path)?.field(), Some("installRoot"));
    Ok(())
}

#[test]
fn valid_instance_config_passes() -> Result<(), ConfigError> {
    let config = InstanceFileConfig::from_json_str(VALID_INSTANCE)?;
    assert_eq!(config.id, "hub");
    assert_eq!(config.kind, InstanceKind::Paper);
    Ok(())
}

#[test]
fn invalid_instance_id_reports_field() -> Result<(), ConfigError> {
    let invalid = VALID_INSTANCE.replace("hub", "Hub");
    let error = InstanceFileConfig::from_json_str(&invalid)
        .err()
        .ok_or_else(|| ConfigError::invalid("test", "expected failure"))?;
    assert_eq!(error.field(), Some("id"));
    Ok(())
}
