use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use base64::Engine;

pub fn ensure_secret_file(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create secret dir: {error}"))?;
    }
    let secret = random_secret()?;
    if !crate::support::private_file::create_private(path, format!("{secret}\n").as_bytes())? {
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("chmod secret: {error}"))?;
    }
    Ok(())
}

pub fn read_secret(path: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map(|value| value.trim_end().to_string())
        .map_err(|error| format!("read secret {path}: {error}"))
}

fn random_secret() -> Result<String, String> {
    let mut bytes = [0_u8; 32];
    fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .map_err(|error| format!("read randomness: {error}"))?;
    Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::ensure_secret_file;

    #[test]
    fn bootstrap_secret_creation_observes_no_broad_mode() -> Result<(), String> {
        let root = std::env::temp_dir().join(format!("lkjmc-secret-{}", std::process::id()));
        let path = root.join("daemon.token");
        let _ = std::fs::remove_dir_all(&root);
        ensure_secret_file(path.to_str().ok_or("temporary path is not UTF-8")?)?;
        let mode = std::fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .permissions()
            .mode()
            & 0o777;
        let content = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        std::fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        assert_eq!(mode, 0o600);
        assert!(!content.trim().is_empty());
        Ok(())
    }
}
