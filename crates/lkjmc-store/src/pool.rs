use std::time::Duration;

use postgres::{Client, Config, NoTls, Transaction};
use r2d2::Pool as R2d2Pool;
use r2d2_postgres::PostgresConnectionManager;

use crate::error::StoreError;

pub type Pool = R2d2Pool<PostgresConnectionManager<NoTls>>;
pub type PooledConnection = r2d2::PooledConnection<PostgresConnectionManager<NoTls>>;

pub fn build(database_url: &str, max_size: u32, ceiling: Duration) -> Result<Pool, StoreError> {
    if ceiling.is_zero() {
        return Err(StoreError::Deadline);
    }
    let mut config: Config = database_url.parse().map_err(StoreError::from)?;
    config.connect_timeout(ceiling);
    configure(&mut config, ceiling);
    let manager = PostgresConnectionManager::new(config, NoTls);
    R2d2Pool::builder()
        .connection_timeout(Duration::from_secs(2))
        .max_size(max_size)
        .build(manager)
        .map_err(|error| StoreError::invalid_state(error.to_string()))
}

pub fn connect_with_deadline(
    database_url: &str,
    remaining: Duration,
) -> Result<Client, StoreError> {
    if remaining.is_zero() {
        return Err(StoreError::Deadline);
    }
    let mut config: Config = database_url.parse().map_err(StoreError::from)?;
    config.connect_timeout(remaining);
    configure(&mut config, remaining);
    config.connect(NoTls).map_err(StoreError::from)
}

pub fn set_deadlines(client: &mut Client, remaining: Duration) -> Result<(), StoreError> {
    if remaining.is_zero() {
        return Err(StoreError::Deadline);
    }
    let milliseconds = milliseconds(remaining);
    client
        .batch_execute(&format!(
            "set statement_timeout = '{milliseconds}ms'; set lock_timeout = '{milliseconds}ms'"
        ))
        .map_err(StoreError::from)
}

pub fn set_lock_timeout(client: &mut Client, timeout: Duration) -> Result<(), StoreError> {
    if timeout.is_zero() {
        return Err(StoreError::Deadline);
    }
    client
        .batch_execute(&format!("set lock_timeout = '{}ms'", milliseconds(timeout)))
        .map_err(StoreError::from)
}

fn configure(config: &mut Config, duration: Duration) {
    let milliseconds = milliseconds(duration);
    config.options(&format!(
        "-c statement_timeout={milliseconds}ms -c lock_timeout={milliseconds}ms"
    ));
}

fn milliseconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis())
        .unwrap_or(u64::MAX)
        .max(1)
}

pub fn connect_single(database_url: &str) -> Result<Client, StoreError> {
    Client::connect(database_url, NoTls).map_err(StoreError::from)
}

pub fn connect(database_url: &str) -> Result<Client, StoreError> {
    connect_single(database_url)
}

pub fn with_transaction<T>(
    client: &mut Client,
    action: impl FnOnce(&mut Transaction<'_>) -> Result<T, StoreError>,
) -> Result<T, StoreError> {
    let mut transaction = client.transaction()?;
    let result = action(&mut transaction)?;
    transaction.commit()?;
    Ok(result)
}
