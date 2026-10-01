use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Clone, Parser)]
pub struct Config {
    #[arg(long, env = "DATABASE_URL", hide_env_values = true)]
    pub database_url: String,
    #[arg(long, env = "LKJMC_BIND", default_value = "127.0.0.1:18091")]
    pub bind: SocketAddr,
    #[arg(
        long,
        env = "LKJMC_PUBLIC_URL",
        default_value = "https://lkjmc.lkjsxc.com"
    )]
    pub public_url: String,
    #[arg(long, env = "LKJMC_STORAGE", default_value = "/var/lib/lkjmc")]
    pub storage: PathBuf,
    #[arg(long, env = "LKJMC_WEB", default_value = "web/dist")]
    pub web: PathBuf,
    #[arg(long, env = "LKJMC_OIDC_ISSUER")]
    pub oidc_issuer: Option<String>,
    #[arg(long, env = "LKJMC_OIDC_CLIENT_ID")]
    pub oidc_client_id: Option<String>,
    #[arg(long, env = "LKJMC_OIDC_SECRET", hide_env_values = true)]
    pub oidc_secret: Option<String>,
    #[arg(long, env = "LKJMC_VOICE_URL")]
    pub voice_url: Option<String>,
    #[arg(long, env = "LKJMC_VOICE_KEY", hide_env_values = true)]
    pub voice_key: Option<String>,
    #[arg(long, env = "LKJMC_VOICE_SECRET", hide_env_values = true)]
    pub voice_secret: Option<String>,
    #[arg(long, env = "LKJMC_DEVELOPMENT", default_value = "false")]
    pub development: bool,
    #[command(subcommand)]
    pub action: Action,
}
#[derive(Clone, clap::Subcommand)]
pub enum Action {
    Api,
    Migrate,
    RegisterServer {
        id: uuid::Uuid,
        name: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        version: String,
        #[arg(long)]
        address: String,
        #[arg(long, default_value = "8192")]
        memory_mib: i32,
        #[arg(long, default_value = "4000")]
        cpu_millis: i32,
        #[arg(long, default_value = "131072")]
        storage_mib: i64,
    },
    Account {
        name: String,
        #[arg(long)]
        admin: bool,
    },
    GrantAdmin {
        account: uuid::Uuid,
    },
    Credential {
        name: String,
        role: String,
        #[arg(long)]
        server: Option<uuid::Uuid>,
        output: PathBuf,
    },
    DevSession {
        account: uuid::Uuid,
        output: PathBuf,
    },
}
impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        let url = reqwest::Url::parse(&self.public_url)?;
        anyhow::ensure!(
            url.path() == "/" && url.query().is_none() && url.fragment().is_none(),
            "public URL must be an origin"
        );
        if self.development {
            anyhow::ensure!(
                self.bind.ip().is_loopback(),
                "development mode requires a loopback listener"
            );
            anyhow::ensure!(
                matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1")),
                "development origin must be local"
            );
        } else {
            anyhow::ensure!(url.scheme() == "https", "production requires HTTPS");
            if matches!(self.action, Action::Api) {
                anyhow::ensure!(
                    self.oidc_issuer
                        .as_ref()
                        .is_some_and(|s| s.starts_with("https://"))
                        && self.oidc_client_id.is_some()
                        && self.oidc_secret.is_some(),
                    "production requires OIDC configuration"
                );
            }
        }
        Ok(())
    }
}
