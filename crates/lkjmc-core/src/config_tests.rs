use super::*;

const VALID_MAIN: &str = r#"{
  "installRoot": "/opt/lkjmc",
  "configRoot": "/etc/lkjmc",
  "dataRoot": "/var/lib/lkjmc",
  "logRoot": "/var/log/lkjmc",
  "socketPath": "/run/lkjmc/daemon.sock",
  "database": {
    "host": "127.0.0.1",
    "port": 5432,
    "database": "lkjmc",
    "user": "lkjmc",
    "secretFile": "/etc/lkjmc/database.secret"
  },
  "network": {
    "defaultLocale": "en",
    "fallbackServer": "hub",
    "onlineMode": true,
    "velocityForwarding": "modern"
  },
  "jars": {
    "root": "/opt/lkjmc/jars",
    "defaultChannel": "stable",
    "userAgent": "lkjmc (+https://github.com/lkjsxc/lkjmc)"
  },
  "runtime": {
    "adapter": "local-process",
    "defaultJavaMemoryMb": 2048,
    "stopTimeoutSeconds": 30
  }
}"#;

const VALID_INSTANCE: &str = r#"{
  "id": "hub",
  "kind": "paper",
  "desiredState": "running",
  "jarRef": "paper:stable",
  "serverPort": 25566,
  "rconPort": 25576,
  "memoryMb": 2048,
  "template": "paper-survival",
  "properties": {"motd": "lkjmc hub"},
  "plugins": {"lkjmcPaper": true},
  "sync": {"playerProfile": true, "location": false}
}"#;

#[test]
fn valid_main_config_passes() -> Result<(), ConfigError> {
    let config = LkjmcConfig::from_json_str(VALID_MAIN)?;
    assert_eq!(config.install_root, "/opt/lkjmc");
    assert_eq!(config.runtime.default_java_memory_mb, 2048);
    Ok(())
}

#[test]
fn invalid_main_config_reports_field() -> Result<(), ConfigError> {
    let invalid = VALID_MAIN.replace("/opt/lkjmc", "opt/lkjmc");
    let error = match LkjmcConfig::from_json_str(&invalid) {
        Ok(_) => return Err(ConfigError::invalid("test", "expected failure")),
        Err(error) => error,
    };
    assert_eq!(error.field(), Some("installRoot"));
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
    let error = match InstanceFileConfig::from_json_str(&invalid) {
        Ok(_) => return Err(ConfigError::invalid("test", "expected failure")),
        Err(error) => error,
    };
    assert_eq!(error.field(), Some("id"));
    Ok(())
}
