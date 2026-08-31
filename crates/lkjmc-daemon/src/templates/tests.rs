use std::fs;

use serde_json::json;

use super::*;

#[test]
fn rendered_secret_file_is_private_on_publication() -> Result<(), String> {
    let root = temp_root("lkjmc-template");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("config/templates")).map_err(|error| error.to_string())?;
    fs::write(root.join("config/templates/custom.json"), template())
        .map_err(|error| error.to_string())?;
    let secret = root.join("forwarding.secret");
    fs::write(&secret, "secret-value\n").map_err(|error| error.to_string())?;
    let state = state(&root);
    let dir = render_instance(
        &state,
        "quartz-world",
        "paper",
        &json!({
            "template":"custom",
            "serverPort":25570,
            "forwardingSecretFile":secret
        }),
    )?;
    let props =
        fs::read_to_string(dir.join("server.properties")).map_err(|error| error.to_string())?;
    assert!(props.contains("difficulty=hard"));
    assert!(props.contains("online-mode=false"));
    assert!(!dir.join("eula.txt").exists());
    let paper = fs::read_to_string(dir.join("config/paper-global.yml"))
        .map_err(|error| error.to_string())?;
    assert!(paper.contains("secret: \"secret-value\""));
    #[cfg(unix)]
    assert_eq!(
        std::os::unix::fs::PermissionsExt::mode(
            &fs::metadata(dir.join("config/paper-global.yml"))
                .map_err(|error| error.to_string())?
                .permissions()
        ) & 0o777,
        0o600
    );
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn renders_complete_velocity_config() -> Result<(), String> {
    let root = temp_root("lkjmc-velocity-template");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("config/templates")).map_err(|error| error.to_string())?;
    let secret = root.join("forwarding.secret");
    fs::write(&secret, "proxy-secret\n").map_err(|error| error.to_string())?;
    let state = state(&root);
    let dir = render_instance(
        &state,
        "front-door",
        "velocity",
        &json!({
            "template":"velocity-modern",
            "serverPort":25565,
            "backendAddresses":{
                "alpha-world":"127.0.0.1:25566",
                "beta-world":"127.0.0.1:25567"
            },
            "defaultBackend":"beta-world",
            "forwardingSecretFile":secret,
            "proxyOnlineMode": false,
            "publicHosts":["lkjsxc.com"]
        }),
    )?;
    let velocity =
        fs::read_to_string(dir.join("velocity.toml")).map_err(|error| error.to_string())?;
    assert!(velocity.contains("config-version = \"2.7\""));
    assert!(velocity.contains("online-mode = false"));
    assert!(velocity.contains("force-key-authentication = false"));
    assert!(velocity.contains("player-info-forwarding-mode = \"modern\""));
    assert!(velocity.contains("alpha-world = \"127.0.0.1:25566\""));
    assert!(velocity.contains("beta-world = \"127.0.0.1:25567\""));
    assert!(velocity.contains("try = [\"beta-world\", \"alpha-world\"]"));
    assert!(velocity.contains("\"lkjsxc.com\" = [\"beta-world\"]"));
    assert!(dir.join("plugins").exists());
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn rejects_unsafe_template_paths() -> Result<(), String> {
    let root = temp_root("lkjmc-template-path");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("config/templates")).map_err(|error| error.to_string())?;
    let state = state(&root);
    let result = render_instance(
        &state,
        "quartz-world",
        "paper",
        &json!({"template":"paper-survival","files":{"../bad":"no"}}),
    );
    assert!(result.is_err());
    let result = render_instance(
        &state,
        "quartz-world",
        "paper",
        &json!({"template":"paper-survival","files":{"eula.txt":"eula=true\n"}}),
    );
    assert!(result.is_err());
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

fn state(root: &std::path::Path) -> AppState {
    AppState::with_config_path(
        None,
        8,
        root.join("config").to_string_lossy().to_string(),
        root.join("logs").to_string_lossy().to_string(),
        root.join("jars").to_string_lossy().to_string(),
        root.join("data").to_string_lossy().to_string(),
        None,
        None,
        None,
    )
}

fn temp_root(prefix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("{prefix}-{}", std::process::id()))
}

fn template() -> &'static str {
    r#"{"properties":{"difficulty":"hard"},"velocityProxy":true}"#
}
