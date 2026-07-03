use postgres::{Client, NoTls, Transaction};
use r2d2::Pool as R2d2Pool;
use r2d2_postgres::PostgresConnectionManager;

use crate::error::StoreError;

pub type Pool = R2d2Pool<PostgresConnectionManager<NoTls>>;
pub type PooledConnection = r2d2::PooledConnection<PostgresConnectionManager<NoTls>>;

pub fn build(database_url: &str, max_size: u32) -> Result<Pool, StoreError> {
    let config = database_url.parse().map_err(StoreError::from)?;
    let manager = PostgresConnectionManager::new(config, NoTls);
    R2d2Pool::builder()
        .max_size(max_size)
        .build(manager)
        .map_err(|error| StoreError::invalid_state(error.to_string()))
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
