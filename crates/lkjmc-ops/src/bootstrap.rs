use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use crate::error::{OpsError, Result};
use crate::fleet::{read_config, FleetSnapshot};
use crate::process::{require_success, run_bounded_owned, CommandSpec};

const MAX_STATUS_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapReceipt {
    pub schema_version: u32,
    pub result: &'static str,
    pub commit: String,
    pub fleet_revision: u64,
    pub instance_ids: Vec<String>,
    pub velocity_instance_id: String,
    pub velocity_status_observed: bool,
}

pub fn after_start(
    config_path: &Path,
    cli_path: &Path,
    expected_commit: &str,
    socket_timeout: Duration,
) -> Result<BootstrapReceipt> {
    if socket_timeout.is_zero() || socket_timeout > Duration::from_secs(300) {
        return Err(OpsError::message(
            "bootstrap socket timeout must be between 1 and 300 seconds",
        ));
    }
    let config = read_config(config_path)?;
    let fleet = FleetSnapshot::from_config(&config)?;
    let socket = Path::new(&config.socket_path);
    wait_for_socket(socket, socket_timeout)?;
    let output = require_success(
        run_bounded_owned(
            &CommandSpec {
                executable: cli_path.to_path_buf(),
                arguments: vec![
                    "--socket".to_string(),
                    config.socket_path.clone(),
                    "--json".to_string(),
                    "status".to_string(),
                ],
                environment: BTreeMap::new(),
                stdin: Vec::new(),
                timeout: Duration::from_secs(30),
                max_output_bytes: MAX_STATUS_BYTES,
            },
            0,
            None,
        )?,
        "daemon status observation",
    )?;
    let status: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| OpsError::context("invalid daemon status JSON", error))?;
    fleet.validate_status(&status, expected_commit)?;
    let velocity = fleet.velocity_entry()?;
    let host = connect_host(&velocity.bind_host)?;
    ping_velocity(host, velocity.port, Duration::from_secs(5))?;
    Ok(BootstrapReceipt {
        schema_version: 1,
        result: "accepted",
        commit: expected_commit.to_string(),
        fleet_revision: fleet.revision,
        instance_ids: fleet
            .instances()
            .map(|instance| instance.id.as_str().to_string())
            .collect(),
        velocity_instance_id: velocity.id.as_str().to_string(),
        velocity_status_observed: true,
    })
}

pub fn ping_velocity(host: IpAddr, port: u16, timeout: Duration) -> Result<Value> {
    let address = SocketAddr::new(host, port);
    let mut stream = TcpStream::connect_timeout(&address, timeout)
        .map_err(|error| OpsError::context("Velocity status connection failed", error))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| OpsError::context("cannot bound Velocity status read", error))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| OpsError::context("cannot bound Velocity status write", error))?;
    let host_text = host.to_string();
    let host_bytes = host_text.as_bytes();
    let mut handshake = Vec::new();
    encode_varint(0, &mut handshake);
    encode_varint(u32::MAX, &mut handshake);
    encode_varint(host_bytes.len() as u32, &mut handshake);
    handshake.extend_from_slice(host_bytes);
    handshake.extend_from_slice(&port.to_be_bytes());
    encode_varint(1, &mut handshake);
    let mut request = Vec::new();
    encode_varint(handshake.len() as u32, &mut request);
    request.extend_from_slice(&handshake);
    request.extend_from_slice(&[1, 0]);
    stream
        .write_all(&request)
        .map_err(|error| OpsError::context("cannot write Velocity status request", error))?;
    let packet_length = read_varint(&mut stream)? as usize;
    if !(3..=MAX_STATUS_BYTES).contains(&packet_length) {
        return Err(OpsError::message(
            "Velocity status packet length is outside its bound",
        ));
    }
    let packet_id = read_varint(&mut stream)?;
    let text_length = read_varint(&mut stream)? as usize;
    if packet_id != 0
        || text_length < 2
        || text_length > packet_length
        || text_length > MAX_STATUS_BYTES
    {
        return Err(OpsError::message("Velocity status response header differs"));
    }
    let mut payload = vec![0_u8; text_length];
    stream
        .read_exact(&mut payload)
        .map_err(|error| OpsError::context("Velocity status response is truncated", error))?;
    let value: Value = serde_json::from_slice(&payload)
        .map_err(|error| OpsError::context("invalid Velocity status JSON", error))?;
    if !value.get("version").is_some_and(Value::is_object)
        || !value.get("players").is_some_and(Value::is_object)
    {
        return Err(OpsError::message("Velocity status payload differs"));
    }
    Ok(value)
}

fn wait_for_socket(path: &Path, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        match fs::symlink_metadata(path) {
            Ok(metadata)
                if metadata.file_type().is_socket() && !metadata.file_type().is_symlink() =>
            {
                return Ok(())
            }
            Ok(_) => {
                return Err(OpsError::message(
                    "daemon socket path exists but is not a Unix socket",
                ))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(OpsError::context("cannot inspect daemon socket", error));
            }
        }
        if Instant::now() >= deadline {
            return Err(OpsError::message(
                "daemon socket did not become ready before its deadline",
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn connect_host(value: &str) -> Result<IpAddr> {
    match value {
        "0.0.0.0" => Ok(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
        "::" => Ok(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)),
        _ => value
            .parse()
            .map_err(|_| OpsError::message("Velocity listener host is not a literal IP address")),
    }
}

fn encode_varint(mut value: u32, output: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn read_varint(input: &mut impl Read) -> Result<u32> {
    let mut value = 0_u32;
    for shift in (0..35).step_by(7) {
        let mut raw = [0_u8; 1];
        input
            .read_exact(&mut raw)
            .map_err(|error| OpsError::context("Velocity status VarInt is truncated", error))?;
        value |= u32::from(raw[0] & 0x7f) << shift;
        if raw[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(OpsError::message("Velocity status VarInt is too long"))
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, TcpListener};

    use super::*;

    #[test]
    fn velocity_status_ping_uses_derived_socket_and_validates_payload() -> Result<()> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .map_err(|error| OpsError::context("cannot bind status fixture", error))?;
        let port = listener
            .local_addr()
            .map_err(|error| OpsError::context("cannot inspect status fixture", error))?
            .port();
        let server = std::thread::spawn(move || -> Result<()> {
            let (mut stream, _) = listener
                .accept()
                .map_err(|error| OpsError::context("status fixture accept failed", error))?;
            let _ = read_varint(&mut stream)?;
            let mut buffer = [0_u8; 1024];
            let _ = stream
                .read(&mut buffer)
                .map_err(|error| OpsError::context("status fixture read failed", error))?;
            let payload =
                br#"{"version":{"name":"fixture","protocol":1},"players":{"max":20,"online":0}}"#;
            let mut packet = Vec::new();
            encode_varint(0, &mut packet);
            encode_varint(payload.len() as u32, &mut packet);
            packet.extend_from_slice(payload);
            let mut response = Vec::new();
            encode_varint(packet.len() as u32, &mut response);
            response.extend_from_slice(&packet);
            stream
                .write_all(&response)
                .map_err(|error| OpsError::context("status fixture write failed", error))?;
            Ok(())
        });
        let response = ping_velocity(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            port,
            Duration::from_secs(2),
        )?;
        assert_eq!(response["players"]["online"], 0);
        server
            .join()
            .map_err(|_| OpsError::message("status fixture thread failed"))??;
        Ok(())
    }
}
