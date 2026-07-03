use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::app::AppState;
use crate::support::instance_helpers::store;

pub fn wait_running(
    state: &AppState,
    client: &mut postgres::Client,
    id: &str,
) -> Result<(), String> {
    let port = server_port(client, id)?;
    let deadline = Instant::now() + Duration::from_secs(1800);
    while Instant::now() < deadline {
        if !crate::support::instance_helpers::runtime_running(state, id)? {
            return Err(format!("instance exited before readiness: {id}"));
        }
        if ready_log(state, id) || tcp_ready(port) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(format!("instance did not become ready: {id}"))
}

fn server_port(client: &mut postgres::Client, id: &str) -> Result<u16, String> {
    let config = store(lkjmc_store::instance::config(client, id))?
        .ok_or_else(|| format!("instance config not found: {id}"))?;
    config
        .get("serverPort")
        .and_then(|value| value.as_u64())
        .and_then(|value| u16::try_from(value).ok())
        .ok_or_else(|| format!("instance server port missing: {id}"))
}

fn tcp_ready(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok()
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
