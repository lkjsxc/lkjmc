use std::path::Path;

use uuid::Uuid;

use super::download_io::{ExpectedChecksum, Hashes};

pub(crate) struct Request<'a> {
    pub(crate) project: &'a str,
    pub(crate) channel: &'a str,
    pub(crate) url: &'a str,
    pub(crate) expected_size: Option<i64>,
    pub(crate) sha256: Option<&'a str>,
}

pub(crate) fn download(
    client: &mut postgres::Client,
    target: &Path,
    request: Request<'_>,
    checksum: ExpectedChecksum<'_>,
) -> Result<Hashes, String> {
    match super::download_io::download(request.url, target, request.expected_size, checksum) {
        Ok(hashes) => Ok(hashes),
        Err(error) => {
            record_failure(client, &request)?;
            Err(error)
        }
    }
}

fn record_failure(client: &mut postgres::Client, request: &Request<'_>) -> Result<(), String> {
    lkjmc_store::asset::insert_download(
        client,
        lkjmc_store::asset::NewAssetDownload {
            id: Uuid::new_v4(),
            asset_id: None,
            asset_kind: "server",
            project: request.project,
            channel: &request.channel.to_ascii_lowercase(),
            url: &sanitized_url(request.url),
            result: "failed",
            sha256: request.sha256,
            size_bytes: request.expected_size,
            error: Some("download failed"),
        },
    )
    .map_err(|error| format!("record download failure: {error}"))
}

fn sanitized_url(url: &str) -> String {
    let Some((scheme, remainder)) = url.split_once("://") else {
        return "redacted-url".to_string();
    };
    let without_query = remainder.split(['?', '#']).next().unwrap_or_default();
    let (authority, path) = without_query
        .split_once('/')
        .map_or((without_query, ""), |(host, path)| (host, path));
    let host = authority.rsplit('@').next().unwrap_or_default();
    if host.is_empty() {
        return "redacted-url".to_string();
    }
    format!("{scheme}://{host}/{path}")
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::sanitized_url;

    #[test]
    fn sanitizes_url_credentials_and_query() {
        assert_eq!(
            sanitized_url("https://user:secret@example.test/jars/server.jar?token=abc#part"),
            "https://example.test/jars/server.jar"
        );
    }

    #[test]
    fn redacts_non_hierarchical_url() {
        assert_eq!(sanitized_url("token:secret"), "redacted-url");
    }
}
