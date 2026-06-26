use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256, Sha512};

pub struct Hashes {
    pub sha256: String,
    pub sha512: String,
}

pub fn download_to(url: &str, target: &Path, expected_size: i64) -> Result<Hashes, String> {
    let response = ureq::get(url)
        .set("User-Agent", crate::downloads::USER_AGENT)
        .call()
        .map_err(|error| format!("download plugin: {error}"))?;
    let mut reader = response.into_reader();
    let mut file = fs::File::create(target).map_err(|error| format!("create plugin: {error}"))?;
    let mut sha256 = Sha256::new();
    let mut sha512 = Sha512::new();
    let mut size = 0_i64;
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        size += i64::try_from(count).map_err(|error| error.to_string())?;
        sha256.update(&buffer[..count]);
        sha512.update(&buffer[..count]);
        file.write_all(&buffer[..count])
            .map_err(|error| error.to_string())?;
    }
    if size != expected_size {
        return Err(format!("download size mismatch: {size} != {expected_size}"));
    }
    Ok(Hashes {
        sha256: format!("{:x}", sha256.finalize()),
        sha512: format!("{:x}", sha512.finalize()),
    })
}

pub fn target_path(
    root: &str,
    source: &str,
    plugin: &str,
    sha: &str,
    file: &str,
) -> Result<PathBuf, String> {
    let short = sha
        .get(0..12)
        .ok_or_else(|| "sha512 too short".to_string())?;
    Ok(Path::new(root)
        .join("plugin")
        .join(source)
        .join(plugin)
        .join(format!("{short}-{}", sanitize(file))))
}

pub fn parent(path: &Path) -> Result<&Path, String> {
    path.parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '.' | '_' | '-' => ch,
            _ => '_',
        })
        .collect()
}
