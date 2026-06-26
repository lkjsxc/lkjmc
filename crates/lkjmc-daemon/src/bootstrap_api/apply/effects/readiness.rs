use std::path::Path;
use std::time::{Duration, Instant};

use crate::app::AppState;

pub fn wait_running(state: &AppState, id: &str) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(1800);
    while Instant::now() < deadline {
        if !crate::instance_helpers::runtime_running(state, id)? {
            return Err(format!("instance exited before readiness: {id}"));
        }
        if ready_log(state, id) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(format!("instance did not become ready: {id}"))
}

fn ready_log(state: &AppState, id: &str) -> bool {
    let path = Path::new(&state.log_root())
        .join("instances")
        .join(id)
        .join("current.log");
    std::fs::read(path).is_ok_and(|bytes| {
        let log = String::from_utf8_lossy(&bytes);
        log.contains("Done (") || log.contains("Started Velocity") || log.contains("Listening on")
    })
}
