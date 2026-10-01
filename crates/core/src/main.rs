use clap::Parser;
use lkjmc_core::{
    App, auth,
    config::{Action, Config},
};
use std::{io::Write, os::unix::fs::OpenOptionsExt};
use uuid::Uuid;

fn secret_file(path: &std::path::Path, value: &str) -> anyhow::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(value.as_bytes())?;
    file.sync_all()?;
    Ok(())
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let config = Config::parse();
    let app = App::connect(config.clone()).await?;
    match config.action {
        Action::Migrate => println!("Migrations applied"),
        Action::Api => {
            let listener = tokio::net::TcpListener::bind(config.bind).await?;
            tracing::info!(address=%config.bind,"lkjmc API ready");
            let worker = tokio::spawn(lkjmc_core::services::maintenance(app.clone()));
            axum::serve(listener, lkjmc_core::router(app))
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await?;
            worker.abort();
        }
        Action::Account { name, admin } => {
            let mut tx = app.db.begin().await?;
            let id = auth::create_account(&mut tx, &name).await?;
            if admin {
                sqlx::query("UPDATE accounts SET administrator=true WHERE id=$1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await?;
            println!("{id}");
        }
        Action::GrantAdmin { account } => {
            let n = sqlx::query(
                "UPDATE accounts SET administrator=true WHERE id=$1 AND merged_into IS NULL",
            )
            .bind(account)
            .execute(&app.db)
            .await?
            .rows_affected();
            anyhow::ensure!(n == 1, "account not found");
            println!("Administrator granted to {account}");
        }
        Action::Credential {
            name,
            role,
            server,
            output,
        } => {
            anyhow::ensure!(
                matches!(role.as_str(), "host" | "proxy" | "official" | "lobby"),
                "invalid service role"
            );
            anyhow::ensure!(
                !matches!(role.as_str(), "official" | "lobby") || server.is_some(),
                "game adapter credentials require a server ID"
            );
            let token = auth::random_token();
            let mut tx = app.db.begin().await?;
            sqlx::query("INSERT INTO service_credentials(id,name,token_hash,role,server_id) VALUES($1,$2,$3,$4,$5)").bind(Uuid::new_v4()).bind(name).bind(auth::hash(&token)).bind(role).bind(server).execute(&mut *tx).await?;
            secret_file(&output, &token)?;
            tx.commit().await?;
            println!("Credential written with mode 0600: {}", output.display());
        }
        Action::DevSession { account, output } => {
            anyhow::ensure!(
                config.development,
                "dev sessions are disabled in production"
            );
            let mut tx = app.db.begin().await?;
            let (token, csrf) = auth::new_session(&mut tx, account).await?;
            secret_file(
                &output,
                &serde_json::to_string(
                    &serde_json::json!({"token":token,"csrf":csrf,"account_id":account}),
                )?,
            )?;
            tx.commit().await?;
            println!("Local session written with mode 0600: {}", output.display());
        }
    }
    Ok(())
}
