use crate::app::AppState;
use crate::runtime::RuntimeObservation;

use super::instance_helpers::{runtime_running, runtime_start, store, write_observation};

const START_ATTEMPTS: usize = 2;
const RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(100);

pub(crate) fn start_runtime(
    state: &AppState,
    client: &mut postgres::Client,
    id: &str,
) -> Result<RuntimeObservation, String> {
    let instance = store(lkjmc_store::instance::get(client, id))?
        .ok_or_else(|| format!("instance not found: {id}"))?;
    let config = store(lkjmc_store::instance::config(client, id))?
        .ok_or_else(|| format!("instance config not found: {id}"))?;
    let work_dir = crate::templates::render_instance(state, id, &instance.kind, &config)?;
    let launch = crate::runtime::instance_launch::launch(state, client, &instance.kind, &config)?;
    let mut failure = "process did not become healthy after start".to_string();
    for attempt in 0..START_ATTEMPTS {
        let observation = runtime_start(
            state,
            id,
            &launch.command,
            &launch.args,
            &launch.env,
            &work_dir,
        )?;
        if observation.healthy && runtime_running(state, id)? {
            write_observation(client, id, &observation)?;
            return Ok(observation);
        }
        let observation = if observation.healthy {
            RuntimeObservation::absent("process absent immediately after start")
        } else {
            observation
        };
        failure = observation.message.clone().unwrap_or(failure);
        write_observation(client, id, &observation)?;
        if attempt + 1 < START_ATTEMPTS {
            std::thread::sleep(retry_backoff(attempt));
        }
    }
    Err(failure)
}

fn retry_backoff(attempt: usize) -> std::time::Duration {
    let shift = u32::try_from(attempt.min(3)).unwrap_or(3);
    RETRY_DELAY
        .saturating_mul(1_u32 << shift)
        .min(std::time::Duration::from_secs(1))
}

#[cfg(test)]
#[path = "runtime_effects_tests.rs"]
mod effect_tests;

#[cfg(test)]
mod tests {
    use super::retry_backoff;
    use std::time::Duration;

    #[test]
    fn retry_backoff_is_exponential_and_bounded() {
        assert_eq!(retry_backoff(0), Duration::from_millis(100));
        assert_eq!(retry_backoff(3), Duration::from_millis(800));
        assert_eq!(retry_backoff(usize::MAX), Duration::from_millis(800));
    }
}
