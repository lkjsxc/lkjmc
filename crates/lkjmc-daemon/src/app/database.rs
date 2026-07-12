use crate::app::admission;

use super::AppState;

impl AppState {
    pub(crate) fn request_database_connection(
        &self,
    ) -> Result<lkjmc_store::pool::PooledConnection, lkjmc_store::error::StoreError> {
        let pool = self.database_pool().ok_or_else(|| {
            lkjmc_store::error::StoreError::invalid_state("database pool is not configured")
        })?;
        let remaining = admission::remaining_request_budget()
            .ok_or(lkjmc_store::error::StoreError::Deadline)?;
        let mut client = match pool.get_timeout(remaining) {
            Ok(client) => client,
            Err(_) if deadline_elapsed() => return Err(lkjmc_store::error::StoreError::Deadline),
            Err(error) => {
                return Err(lkjmc_store::error::StoreError::invalid_state(
                    error.to_string(),
                ));
            }
        };
        let remaining = admission::remaining_request_budget()
            .ok_or(lkjmc_store::error::StoreError::Deadline)?;
        lkjmc_store::pool::set_deadlines(&mut client, remaining)?;
        Ok(client)
    }
}

fn deadline_elapsed() -> bool {
    admission::remaining_request_budget().is_some_and(|remaining| remaining.is_zero())
}
