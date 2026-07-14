use std::time::{Duration, Instant};

use crate::app::admission;

use super::AppState;

impl AppState {
    pub(crate) fn request_database_connection(
        &self,
    ) -> Result<lkjmc_store::pool::PooledConnection, lkjmc_store::error::StoreError> {
        if self.database_pool().is_none() {
            return Err(lkjmc_store::error::StoreError::invalid_state(
                "database pool is not configured",
            ));
        }
        let remaining = admission::remaining_request_budget()
            .ok_or(lkjmc_store::error::StoreError::Deadline)?;
        self.request_database_connection_with_budget(remaining)
    }

    pub(crate) fn request_database_connection_with_budget(
        &self,
        budget: Duration,
    ) -> Result<lkjmc_store::pool::PooledConnection, lkjmc_store::error::StoreError> {
        if budget.is_zero() {
            return Err(lkjmc_store::error::StoreError::Deadline);
        }
        let started = Instant::now();
        let pool = self.database_pool().ok_or_else(|| {
            lkjmc_store::error::StoreError::invalid_state("database pool is not configured")
        })?;
        let remaining =
            admission::remaining_request_budget().map_or(budget, |outer| outer.min(budget));
        let mut client = match pool.get_timeout(remaining) {
            Ok(client) => client,
            Err(_) if remaining.is_zero() || deadline_elapsed() => {
                return Err(lkjmc_store::error::StoreError::Deadline);
            }
            Err(error) => {
                return Err(lkjmc_store::error::StoreError::invalid_state(
                    error.to_string(),
                ));
            }
        };
        let remaining = budget.saturating_sub(started.elapsed());
        let remaining =
            admission::remaining_request_budget().map_or(remaining, |outer| outer.min(remaining));
        lkjmc_store::pool::set_deadlines(&mut client, remaining)?;
        #[cfg(test)]
        if let Some(timeout) = self.test_lock_timeout() {
            lkjmc_store::pool::set_lock_timeout(&mut client, timeout)?;
        }
        Ok(client)
    }

    #[cfg(test)]
    pub(crate) fn set_test_lock_timeout(&self, timeout: Duration) -> Result<(), String> {
        self.config
            .write()
            .map_err(|_| "config lock poisoned".to_string())?
            .test_lock_timeout = Some(timeout);
        Ok(())
    }

    #[cfg(test)]
    fn test_lock_timeout(&self) -> Option<Duration> {
        self.option(|config| config.test_lock_timeout)
    }
}

fn deadline_elapsed() -> bool {
    admission::remaining_request_budget().is_some_and(|remaining| remaining.is_zero())
}
