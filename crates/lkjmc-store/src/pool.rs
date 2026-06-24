use postgres::{Client, NoTls};

use crate::error::StoreError;

pub fn connect(database_url: &str) -> Result<Client, StoreError> {
    Client::connect(database_url, NoTls).map_err(StoreError::from)
}
