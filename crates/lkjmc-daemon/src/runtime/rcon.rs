use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde_json::Value;

const AUTH: i32 = 3;
const COMMAND: i32 = 2;

pub fn stop_from_config(config: &Value) -> Result<(), String> {
    let Some(rcon) = config.get("rcon") else {
        return Ok(());
    };
    let host = rcon
        .get("host")
        .and_then(Value::as_str)
        .unwrap_or("127.0.0.1");
    let port = rcon
        .get("port")
        .and_then(Value::as_u64)
        .ok_or_else(|| "rcon.port is required".to_string())?;
    let password = rcon
        .get("password")
        .and_then(Value::as_str)
        .ok_or_else(|| "rcon.password is required".to_string())?;
    send_stop(
        host,
        u16::try_from(port).map_err(|error| error.to_string())?,
        password,
    )
}

fn send_stop(host: &str, port: u16, password: &str) -> Result<(), String> {
    let address = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("resolve rcon: {error}"))?
        .next()
        .ok_or_else(|| "rcon host resolved no addresses".to_string())?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
        .map_err(|error| format!("connect rcon: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("set rcon read timeout: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("set rcon write timeout: {error}"))?;
    send_packet(&mut stream, 1, AUTH, password)?;
    let auth = read_packet(&mut stream)?;
    if auth.id == -1 {
        return Err("rcon authentication failed".to_string());
    }
    send_packet(&mut stream, 2, COMMAND, "stop")?;
    let _ = read_packet(&mut stream)?;
    Ok(())
}

fn send_packet(stream: &mut TcpStream, id: i32, kind: i32, body: &str) -> Result<(), String> {
    let body_bytes = body.as_bytes();
    let length = i32::try_from(8 + body_bytes.len() + 2).map_err(|error| error.to_string())?;
    stream
        .write_all(&length.to_le_bytes())
        .and_then(|_| stream.write_all(&id.to_le_bytes()))
        .and_then(|_| stream.write_all(&kind.to_le_bytes()))
        .and_then(|_| stream.write_all(body_bytes))
        .and_then(|_| stream.write_all(&[0, 0]))
        .map_err(|error| format!("write rcon packet: {error}"))
}

fn read_packet(stream: &mut TcpStream) -> Result<RconPacket, String> {
    let length = read_i32(stream)?;
    if length < 10 {
        return Err("short rcon packet".to_string());
    }
    let mut payload = vec![0_u8; usize::try_from(length).map_err(|error| error.to_string())?];
    stream
        .read_exact(&mut payload)
        .map_err(|error| format!("read rcon packet: {error}"))?;
    let id = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    Ok(RconPacket { id })
}

fn read_i32(stream: &mut TcpStream) -> Result<i32, String> {
    let mut bytes = [0_u8; 4];
    stream
        .read_exact(&mut bytes)
        .map_err(|error| format!("read rcon length: {error}"))?;
    Ok(i32::from_le_bytes(bytes))
}

struct RconPacket {
    id: i32,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn missing_rcon_config_is_noop() {
        assert_eq!(stop_from_config(&json!({})), Ok(()));
    }
}
