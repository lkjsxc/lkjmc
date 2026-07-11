use crate::app::AppState;
use crate::runtime::RuntimeObservation;

use super::instance_helpers::{runtime_running, runtime_start, store, write_observation};

const START_ATTEMPTS: usize = 2;

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
    for _ in 0..START_ATTEMPTS {
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
    }
    Err(failure)
}
