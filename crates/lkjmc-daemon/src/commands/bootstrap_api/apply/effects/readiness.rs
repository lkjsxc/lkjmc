use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::app::AppState;
use crate::support::instance_helpers::store;

const BOOKKEEPING_RESERVE: Duration = Duration::from_secs(5);

pub fn wait_running(state: &AppState, id: &str, port: u16) -> Result<(), String> {
    let remaining = crate::app::remaining_request_budget()
        .unwrap_or(crate::command_lifecycle::NETWORK_APPLY_DEADLINE)
        .min(crate::command_lifecycle::NETWORK_APPLY_DEADLINE);
    let deadline = Instant::now() + wait_budget(remaining)?;
    while Instant::now() < deadline {
        if !crate::support::instance_helpers::runtime_running(state, id)? {
            return Err(format!("instance exited before readiness: {id}"));
        }
        if ready_log(state, id) && tcp_ready(port) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(format!("instance did not become ready: {id}"))
}

pub(crate) fn server_port(client: &mut postgres::Client, id: &str) -> Result<u16, String> {
    let config = store(lkjmc_store::instance::config(client, id))?
        .ok_or_else(|| format!("instance config not found: {id}"))?;
    config
        .get("serverPort")
        .and_then(|value| value.as_u64())
        .and_then(|value| u16::try_from(value).ok())
        .ok_or_else(|| format!("instance server port missing: {id}"))
}

fn wait_budget(remaining: Duration) -> Result<Duration, String> {
    remaining
        .checked_sub(BOOKKEEPING_RESERVE)
        .filter(|budget| !budget.is_zero())
        .ok_or_else(|| "network apply deadline has no readiness budget".to_string())
}

fn tcp_ready(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok()
}

fn ready_log(state: &AppState, id: &str) -> bool {
    let path = Path::new(&state.log_root()).join(id).join("current.log");
    std::fs::read(path).is_ok_and(|bytes| {
        let log = String::from_utf8_lossy(&bytes);
        log.contains(&format!("lkjmc instance {id}\n"))
            && (log.contains("Done (")
                || log.contains("Started Velocity")
                || log.contains("Listening on"))
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{ready_log, wait_budget};
    use crate::app::AppState;

    #[test]
    fn readiness_reserves_time_for_durable_bookkeeping() -> Result<(), String> {
        assert!(wait_budget(Duration::from_secs(5)).is_err());
        assert_eq!(
            wait_budget(Duration::from_secs(12))?,
            Duration::from_secs(7)
        );
        Ok(())
    }

    #[test]
    fn bootstrap_effects_truthful() -> Result<(), String> {
        let root =
            std::env::temp_dir().join(format!("lkjmc-ready-{}", uuid::Uuid::new_v4().simple()));
        let instance_id = format!("hub-{}", uuid::Uuid::new_v4().simple());
        let log = root.join(&instance_id).join("current.log");
        std::fs::create_dir_all(log.parent().ok_or("log parent missing")?)
            .map_err(|error| error.to_string())?;
        let state = AppState::with_config_path(
            None,
            1,
            String::new(),
            root.to_string_lossy().into(),
            String::new(),
            String::new(),
            None,
            None,
            None,
        );
        std::fs::write(&log, "Done (0.1s)!\n").map_err(|error| error.to_string())?;
        assert!(!ready_log(&state, &instance_id));
        std::fs::write(
            &log,
            format!("lkjmc instance {instance_id}\nDone (0.1s)!\n"),
        )
        .map_err(|error| error.to_string())?;
        assert!(ready_log(&state, &instance_id));
        std::fs::remove_dir_all(root).map_err(|error| error.to_string())
    }
}
