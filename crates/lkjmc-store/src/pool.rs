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

pub fn with_search_path(database_url: &str, schema: &str) -> Result<String, StoreError> {
    if !schema
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(StoreError::invalid_state("invalid PostgreSQL schema name"));
    }
    Ok(with_parameter(
        database_url,
        "options",
        &format!("-c search_path={schema},public"),
    ))
}

pub fn with_application_name(database_url: &str, application_name: &str) -> String {
    with_parameter(database_url, "application_name", application_name)
}

fn configure(config: &mut Config, duration: Duration) {
    let milliseconds = milliseconds(duration);
    let deadlines =
        format!("-c statement_timeout={milliseconds}ms -c lock_timeout={milliseconds}ms");
    let options = match config.get_options() {
        Some(existing) => format!("{existing} {deadlines}"),
        None => deadlines,
    };
    config.options(&options);
}

fn with_parameter(database_url: &str, name: &str, value: &str) -> String {
    let separator = if database_url.contains('?') { '&' } else { '?' };
    format!("{database_url}{separator}{name}={}", encode(value))
}

fn encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                encoded.push('%');
                encoded.push(char::from(HEX[usize::from(byte >> 4)]));
                encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
            }
        }
    }
    encoded
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
