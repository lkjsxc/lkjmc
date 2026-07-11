use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde_json::json;

use super::{private_config, stop_from_config};

#[test]
fn missing_rcon_config_is_noop() {
    assert_eq!(stop_from_config(&json!({})), Ok(()));
}

#[test]
fn inline_rcon_password_is_rejected() -> Result<(), String> {
    let error = match stop_from_config(&json!({"rcon":{"port":25575,"password":"bad"}})) {
        Err(error) => error,
        Ok(()) => return Err("inline RCON password was accepted".to_string()),
    };
    assert!(error.contains("passwordFile"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn rcon_password_file_is_private_under_permissive_umask() -> Result<(), String> {
    let root = std::env::var_os("LKJMC_RCON_UMASK_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("lkjmc-rcon-{}", std::process::id())));
    if std::env::var_os("LKJMC_RCON_UMASK_CHILD").is_some() {
        let config = private_config(
            root.to_str().ok_or("temporary root is not UTF-8")?,
            "hub",
            &json!({"port":25575,"password":"not-in-json"}),
        )?;
        assert!(config.get("password").is_none());
        assert!(config["passwordFile"].is_string());
        assert!(!serde_json::to_string(&config)
            .map_err(|error| error.to_string())?
            .contains("not-in-json"));
        return Ok(());
    }
    let _ = fs::remove_dir_all(&root);
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let status = std::process::Command::new("sh")
        .args(["-c", "umask 000; exec \"$@\"", "sh"])
        .arg(executable)
        .args([
            "--exact",
            "runtime::rcon::tests::rcon_password_file_is_private_under_permissive_umask",
            "--nocapture",
        ])
        .env("LKJMC_RCON_UMASK_CHILD", "1")
        .env("LKJMC_RCON_UMASK_ROOT", &root)
        .status()
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err("permissive-umask RCON child failed".to_string());
    }
    let secret = root.join("instances/hub.rcon-password");
    let mode = fs::metadata(&secret)
        .map_err(|error| error.to_string())?
        .permissions()
        .mode()
        & 0o777;
    let content = fs::read_to_string(&secret).map_err(|error| error.to_string())?;
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    assert_eq!(mode, 0o600);
    assert_eq!(content, "not-in-json");
    Ok(())
}
