use std::fs;
use std::path::Path;

use lkjmc_core::config::LkjmcConfig;

const DEFAULT_CONFIG: &str = "/etc/lkjmc/lkjmc.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileConfigValues {
    pub socket: String,
    pub database_url: String,
    pub config_root: String,
    pub log_root: String,
    pub jar_root: String,
    pub data_root: String,
}

pub fn default_path() -> Option<String> {
    Path::new(DEFAULT_CONFIG)
        .exists()
        .then_some(DEFAULT_CONFIG.to_string())
}

pub fn load(path: &str) -> Result<FileConfigValues, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("read config {path}: {error}"))?;
    let config = LkjmcConfig::from_json_str(&content).map_err(|error| error.to_string())?;
    let secret = fs::read_to_string(&config.database.secret_file)
        .map_err(|error| format!("read database secret: {error}"))?;
    let database_url = database_url(&config, secret.trim_end());
    Ok(FileConfigValues {
        socket: config.socket_path,
        database_url,
        config_root: config.config_root,
        log_root: child(&config.log_root, "instances")?,
        jar_root: config.jars.root,
        data_root: child(&config.data_root, "instances")?,
    })
}

fn child(root: &str, name: &str) -> Result<String, String> {
    Path::new(root)
        .join(name)
        .to_str()
        .map(ToString::to_string)
        .ok_or_else(|| format!("invalid path {root}/{name}"))
}

fn database_url(config: &LkjmcConfig, password: &str) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.user,
        password,
        config.database.host,
        config.database.port,
        config.database.database
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_config_without_printing_secret() -> Result<(), String> {
        let root = std::env::temp_dir().join(format!("lkjmc-config-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let secret = root.join("db.secret");
        fs::write(&secret, "pw\n").map_err(|error| error.to_string())?;
        let config = root.join("lkjmc.json");
        fs::write(&config, json(&root, &secret)).map_err(|error| error.to_string())?;
        let config_path = config.to_str().ok_or("invalid config path")?;
        let root_path = root.to_str().ok_or("invalid root path")?;
        let loaded = load(config_path)?;
        assert_eq!(loaded.socket, child(root_path, "daemon.sock")?);
        assert!(loaded
            .database_url
            .contains("postgres://lkjmc:pw@127.0.0.1:5432/lkjmc"));
        assert!(loaded.log_root.ends_with("logs/instances"));
        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    fn json(root: &Path, secret: &Path) -> String {
        format!(
            r#"{{
  "installRoot": "{0}",
  "configRoot": "{0}",
  "dataRoot": "{0}/data",
  "logRoot": "{0}/logs",
  "socketPath": "{0}/daemon.sock",
  "database": {{"host":"127.0.0.1","port":5432,"database":"lkjmc","user":"lkjmc","secretFile":"{1}"}},
  "network": {{"defaultLocale":"en","fallbackServer":"hub","onlineMode":true,"velocityForwarding":"modern"}},
  "jars": {{"root":"{0}/jars","defaultChannel":"stable","userAgent":"lkjmc (+https://github.com/lkjsxc/lkjmc)"}},
  "runtime": {{"adapter":"local-process","defaultJavaMemoryMb":1024,"stopTimeoutSeconds":30}}
}}"#,
            root.display(),
            secret.display()
        )
    }
}
