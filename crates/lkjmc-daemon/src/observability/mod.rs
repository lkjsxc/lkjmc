pub(crate) mod api;
pub(crate) mod health;
pub(crate) mod metrics;

use std::time::Duration;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};

use crate::app::AppState;

pub(crate) fn command_completed(
    state: &AppState,
    request: &CommandEnvelope,
    response: &CommandResponse,
    elapsed: Duration,
) {
    state.metrics().request(response.ok, elapsed);
    let event = match lkjmc_store::observability::command_event(request, response) {
        Ok(value) => value,
        Err(_) => return,
    };
    if let Ok(line) = serde_json::to_string(&event) {
        eprintln!("{line}");
    }
    let Some(_) = state.database_url() else {
        return;
    };
    match state.request_database_connection().and_then(|mut client| {
        lkjmc_store::observability::record_command_event(&mut *client, &event)
    }) {
        Ok(_) => {}
        Err(_) => state.metrics().database_error(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::{Duration, Instant};

    use lkjmc_core::observability::{
        Component, EventEnvelope, EventKind, Outcome, Severity, Surface,
    };

    #[test]
    fn overhead_budget() -> Result<(), String> {
        let started = Instant::now();
        let mut bytes = 0;
        for _ in 0..10_000 {
            let event = EventEnvelope::new(
                Severity::Info,
                Component::Daemon,
                EventKind::ReadinessChecked,
                None,
                None,
                None,
                "daemon",
                "lkjmc-daemon",
                Surface::Internal,
                Outcome::Succeeded,
                None,
                BTreeMap::new(),
                "daemon-local",
            )?;
            bytes += serde_json::to_vec(&event)
                .map_err(|error| error.to_string())?
                .len();
        }
        assert!(bytes > 1_000_000);
        assert!(started.elapsed() < Duration::from_secs(2));
        Ok(())
    }
}
