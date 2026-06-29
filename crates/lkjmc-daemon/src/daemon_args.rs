use std::{env, fs};

#[derive(Debug, Clone)]
pub struct DaemonArgs {
    pub socket: String,
    pub http: Option<String>,
    pub http_token: Option<String>,
    pub database_url: Option<String>,
    pub config_root: String,
    pub log_root: String,
    pub jar_root: String,
    pub data_root: String,
    pub config_path: Option<String>,
}

pub fn parse(values: Vec<String>) -> Result<DaemonArgs, String> {
    let mut args = defaults(&values)?;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--socket" => set(
                &mut args.socket,
                value_after(&values, index, "--socket")?,
                &mut index,
            ),
            "--config" => {
                let _ = value_after(&values, index, "--config")?;
                index += 2;
            }
            "--config-root" => set(
                &mut args.config_root,
                value_after(&values, index, "--config-root")?,
                &mut index,
            ),
            "--http" => {
                let value = value_after(&values, index, "--http")?;
                args.http = (value != "none").then_some(value);
                index += 2;
            }
            "--http-token" => set_opt(
                &mut args.http_token,
                value_after(&values, index, "--http-token")?,
                &mut index,
            ),
            "--http-token-file" => {
                let path = value_after(&values, index, "--http-token-file")?;
                args.http_token = Some(read_secret(&path)?);
                index += 2;
            }
            "--database-url" => set_opt(
                &mut args.database_url,
                value_after(&values, index, "--database-url")?,
                &mut index,
            ),
            "--log-root" => set(
                &mut args.log_root,
                value_after(&values, index, "--log-root")?,
                &mut index,
            ),
            "--jar-root" => set(
                &mut args.jar_root,
                value_after(&values, index, "--jar-root")?,
                &mut index,
            ),
            "--data-root" => set(
                &mut args.data_root,
                value_after(&values, index, "--data-root")?,
                &mut index,
            ),
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(args)
}

fn defaults(values: &[String]) -> Result<DaemonArgs, String> {
    let config_path = requested_config(values)?;
    let mut args = DaemonArgs {
        socket: "/run/lkjmc/daemon.sock".to_string(),
        http: Some("127.0.0.1:8765".to_string()),
        http_token: None,
        database_url: env::var("LKJMC_DATABASE_URL").ok(),
        config_root: "/etc/lkjmc".to_string(),
        log_root: "/var/log/lkjmc/instances".to_string(),
        jar_root: "/opt/lkjmc/jars".to_string(),
        data_root: "/var/lib/lkjmc/instances".to_string(),
        config_path,
    };
    if let Some(config_path) = &args.config_path {
        let config = crate::daemon_config::load(config_path)?;
        args.socket = config.socket;
        args.database_url = Some(config.database_url);
        args.config_root = config.config_root;
        args.log_root = config.log_root;
        args.jar_root = config.jar_root;
        args.data_root = config.data_root;
    }
    Ok(args)
}

fn set(target: &mut String, value: String, index: &mut usize) {
    *target = value;
    *index += 2;
}

fn set_opt(target: &mut Option<String>, value: String, index: &mut usize) {
    *target = Some(value);
    *index += 2;
}

fn read_secret(path: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map(|value| value.trim_end().to_string())
        .map_err(|error| format!("read secret {path}: {error}"))
}

fn requested_config(values: &[String]) -> Result<Option<String>, String> {
    for (index, value) in values.iter().enumerate() {
        if value == "--config" {
            return value_after(values, index, "--config").map(Some);
        }
    }
    Ok(crate::daemon_config::default_path())
}

fn value_after(values: &[String], index: usize, flag: &str) -> Result<String, String> {
    values
        .get(index + 1)
        .cloned()
        .ok_or_else(|| format!("missing value for {flag}"))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn http_api_token_file_trailing_newline_is_trimmed() {
        let path = std::env::temp_dir().join(format!(
            "lkjmc-http-token-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::write(&path, "AbCdEFghIJ09+/==\n").expect("write token");
        let args = parse(vec![
            "--http-token-file".to_string(),
            path.to_string_lossy().into_owned(),
        ])
        .expect("parse args");
        fs::remove_file(path).ok();
        assert_eq!(args.http_token.as_deref(), Some("AbCdEFghIJ09+/=="));
    }
}
