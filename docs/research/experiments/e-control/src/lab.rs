use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio_postgres::{Client, NoTls};
use url::Url;

#[derive(Clone, Copy)]
pub struct Counts {
    pub operations: i64,
    pub journal: i64,
    pub attempts: i64,
    pub effects: i64,
}

pub fn disposable_url() -> Option<String> {
    let value = std::env::var("LKJMC_LAB_POSTGRES_URL").ok()?;
    if std::env::var("LKJMC_LAB_POSTGRES_DISPOSABLE").as_deref() != Ok("1") || !usable_url(&value) {
        None
    } else {
        Some(value)
    }
}

fn usable_url(value: &str) -> bool {
    let Ok(url) = Url::parse(value) else { return false };
    let name = url.path().trim_start_matches('/');
    let host = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"))
        || (url.host_str() == Some("postgres")
            && std::env::var("LKJMC_E_CONTROL_COMPOSE").as_deref() == Ok("1"));
    matches!(url.scheme(), "postgres" | "postgresql")
        && url.query().is_none()
        && url.fragment().is_none()
        && host
        && name.starts_with("lkjmc_lab_")
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub fn schema() -> String {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |v| v.as_nanos());
    format!("lkjmc_lab_e_control_{}_{}", std::process::id(), nonce)
}

pub async fn connect(url: &str) -> Result<(Client, tokio::task::JoinHandle<Result<(), tokio_postgres::Error>>), ()> {
    let (client, connection) = tokio_postgres::connect(url, NoTls).await.map_err(|_| ())?;
    Ok((client, tokio::spawn(connection)))
}

pub async fn wait_database(url: &str) -> Result<(), ()> {
    for _ in 0..120 {
        if let Ok((client, connection)) = connect(url).await {
            let healthy = client.query_one("SELECT 1", &[]).await.is_ok();
            connection.abort();
            if healthy { return Ok(()); }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(())
}

pub async fn setup(url: &str, schema: &str) -> Result<(), ()> {
    let (client, connection) = connect(url).await?;
    let sql = format!(
        "CREATE SCHEMA {schema};\
         CREATE TABLE {schema}.operations (request_id text PRIMARY KEY, state text NOT NULL);\
         CREATE TABLE {schema}.journal (request_id text PRIMARY KEY, state text NOT NULL);\
         CREATE TABLE {schema}.effect_attempts (request_id text PRIMARY KEY, state text NOT NULL);\
         CREATE TABLE {schema}.effects (request_id text PRIMARY KEY, state text NOT NULL);\
         CREATE TABLE {schema}.deadline (request_id text PRIMARY KEY)"
    );
    client.batch_execute(&sql).await.map_err(|_| ())?;
    connection.abort();
    Ok(())
}

pub async fn reset(url: &str, schema: &str) -> Result<(), ()> {
    let (client, connection) = connect(url).await?;
    client.batch_execute(&format!(
        "TRUNCATE {schema}.operations, {schema}.journal, {schema}.effect_attempts, {schema}.effects, {schema}.deadline"
    )).await.map_err(|_| ())?;
    connection.abort();
    Ok(())
}

pub async fn counts(url: &str, schema: &str) -> Result<Counts, ()> {
    let (client, connection) = connect(url).await?;
    let result = Counts {
        operations: table_count(&client, schema, "operations").await?,
        journal: table_count(&client, schema, "journal").await?,
        attempts: table_count(&client, schema, "effect_attempts").await?,
        effects: table_count(&client, schema, "effects").await?,
    };
    connection.abort();
    Ok(result)
}

async fn table_count(client: &Client, schema: &str, table: &str) -> Result<i64, ()> {
    client.query_one(&format!("SELECT count(*) FROM {schema}.{table}"), &[]).await
        .map(|row| row.get(0)).map_err(|_| ())
}

pub async fn drop_schema(url: &str, schema: &str) -> Result<(), ()> {
    let (client, connection) = connect(url).await?;
    let result = client.batch_execute(&format!("DROP SCHEMA IF EXISTS {schema} CASCADE")).await.map_err(|_| ());
    connection.abort();
    result
}

pub async fn deadline(url: &str, schema: &str) -> Result<&'static str, ()> {
    let (client, connection) = connect(url).await?;
    let cancel = client.cancel_token();
    let sql = format!("SELECT pg_sleep(0.05); INSERT INTO {schema}.deadline VALUES ('must-not-appear')");
    let query = client.batch_execute(&sql);
    let cancel_query = async move {
        tokio::time::sleep(Duration::from_millis(2)).await;
        cancel.cancel_query(NoTls).await.map_err(|_| ())
    };
    let (query, cancelled) = tokio::join!(query, cancel_query);
    let absent = client.query_one(&format!("SELECT count(*) FROM {schema}.deadline"), &[]).await
        .map(|row| row.get::<_, i64>(0) == 0).map_err(|_| ())?;
    connection.abort();
    if query.is_err() && cancelled.is_ok() && absent { Ok("cancelled-and-absent") } else { Err(()) }
}
