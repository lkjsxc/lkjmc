use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use base64::Engine;

pub fn ensure_secret_file(path: &str) -> Result<(), String> {
    if Path::new(path).exists() {
        return Ok(());
    }
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create secret dir: {error}"))?;
    }
    let secret = random_secret()?;
    fs::write(path, format!("{secret}\n")).map_err(|error| format!("write secret: {error}"))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|error| format!("chmod secret: {error}"))
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
