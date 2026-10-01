mod client;
mod config;
mod incus;
mod probe;
mod retention;
mod state;
mod worker;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    config: PathBuf,
    /// Validate local configuration without taking a job or changing an instance.
    #[arg(long)]
    check: bool,
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = Args::parse();
    let config = config::Config::load(&args.config)?;
    if args.check {
        println!(
            "Local configuration is valid; VM and network readiness require host verification."
        );
        return Ok(());
    }
    worker::Worker::new(config)?.run().await
}
