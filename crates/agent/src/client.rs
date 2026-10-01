use anyhow::{Result, ensure};
use futures::StreamExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Arc, time::Duration};
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    origin: reqwest::Url,
    token: Arc<String>,
}
impl Client {
    pub fn new(origin: &str, token_file: &Path) -> Result<Self> {
        let token = std::fs::read_to_string(token_file)?.trim().to_string();
        ensure!(token.len() >= 32, "Missing host credential");
        Ok(Self {
            http: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(120))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
            origin: reqwest::Url::parse(origin)?,
            token: Arc::new(token),
        })
    }
    pub async fn request(&self, path: &str, body: Option<Value>) -> Result<Value> {
        ensure!(path.starts_with("/internal/v1/"), "Invalid internal route");
        let request = if let Some(body) = body {
            self.http.post(self.origin.join(path)?).json(&body)
        } else {
            self.http.get(self.origin.join(path)?)
        };
        let response = request
            .bearer_auth(self.token.as_str())
            .timeout(Duration::from_secs(20))
            .send()
            .await?;
        let status = response.status();
        let value: Value = response.json().await?;
        ensure!(
            status.is_success(),
            "Core {}: {}",
            status.as_u16(),
            value["error"]["message"]
                .as_str()
                .unwrap_or("request rejected")
        );
        Ok(value)
    }
    pub async fn ack(
        &self,
        job: &Value,
        state: &str,
        result: Value,
        message: Option<&str>,
    ) -> Result<()> {
        self.request(&format!("/internal/v1/jobs/{}/ack",job["id"].as_str().unwrap()),Some(json!({"lease_token":job["lease_token"],"state":state,"result":result,"progress":message.map(|m|json!({"message":m})).unwrap_or(json!({})),"error":if state=="failed"{message}else{None}}))).await?;
        Ok(())
    }
    pub async fn download(
        &self,
        url: &str,
        internal: bool,
        expected: &str,
        limit: u64,
        path: &Path,
    ) -> Result<()> {
        if path.exists() && file_hash(path).await? == expected {
            return Ok(());
        }
        let mut request = self.http.get(if internal {
            ensure!(
                url.starts_with("/internal/v1/artifacts/")
                    || url.starts_with("/internal/v1/official-backups/"),
                "Invalid artifact path"
            );
            self.origin.join(url)?
        } else {
            let u = reqwest::Url::parse(url)?;
            ensure!(u.scheme() == "https", "Preset requires HTTPS");
            u
        });
        if internal {
            request = request.bearer_auth(self.token.as_str());
        }
        let response = request
            .timeout(Duration::from_secs(3600))
            .send()
            .await?
            .error_for_status()?;
        ensure!(
            response.status().is_success(),
            "Download did not return an artifact"
        );
        if let Some(size) = response.content_length() {
            ensure!(size <= limit, "Artifact is too large");
        }
        let temporary = path.with_extension("part");
        let mut file = tokio::fs::File::create(&temporary).await?;
        let mut sha = Sha256::new();
        let mut size = 0u64;
        let mut stream = response.bytes_stream();
        while let Some(bytes) = stream.next().await {
            let bytes = bytes?;
            size += bytes.len() as u64;
            ensure!(size <= limit, "Artifact exceeded its size limit");
            sha.update(&bytes);
            file.write_all(&bytes).await?;
        }
        ensure!(
            hex::encode(sha.finalize()) == expected,
            "Artifact SHA256 mismatch"
        );
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(temporary, path).await?;
        std::fs::File::open(path.parent().unwrap())?.sync_all()?;
        Ok(())
    }
}
pub async fn file_hash(path: &Path) -> Result<String> {
    use tokio::io::AsyncReadExt;
    let mut file = tokio::fs::File::open(path).await?;
    let mut buffer = vec![0u8; 1048576];
    let mut hash = Sha256::new();
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(hex::encode(hash.finalize()))
}
