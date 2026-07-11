use std::path::{Path, PathBuf};

pub use crate::assets::download_io::Hashes;

pub fn download_to(
    url: &str,
    target: &Path,
    expected_size: i64,
    checksum: crate::assets::download_io::ExpectedChecksum<'_>,
) -> Result<Hashes, String> {
    crate::assets::download_io::download(url, target, Some(expected_size), checksum)
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

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '.' | '_' | '-' => ch,
            _ => '_',
        })
        .collect()
}
