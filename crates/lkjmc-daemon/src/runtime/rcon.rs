use std::fs;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};

const AUTH: i32 = 3;
const COMMAND: i32 = 2;

pub(crate) fn private_config(config_root: &str, id: &str, rcon: &Value) -> Result<Value, String> {
    let password = rcon
        .get("password")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "rcon.password is required".to_string())?;
    let port = rcon
        .get("port")
        .and_then(Value::as_u64)
        .ok_or_else(|| "rcon.port is required".to_string())?;
    let parent = Path::new(config_root).join("instances");
    fs::create_dir_all(&parent).map_err(|error| format!("create rcon secret dir: {error}"))?;
    #[cfg(unix)]
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700))
        .map_err(|error| format!("chmod rcon secret dir: {error}"))?;
    let password_file = parent.join(format!("{id}.rcon-password"));
    crate::support::private_file::replace_private(&password_file, password.as_bytes())?;
    Ok(json!({
        "host": rcon.get("host").and_then(Value::as_str).unwrap_or("127.0.0.1"),
        "port": port,
        "passwordFile": password_file
    }))
}

pub fn stop_from_config(config: &Value) -> Result<(), String> {
    let Some(rcon) = config.get("rcon") else {
        return Ok(());
    };
    if rcon.get("password").is_some() {
        return Err("rcon.password is forbidden; use rcon.passwordFile".to_string());
    }
    let host = rcon
        .get("host")
        .and_then(Value::as_str)
        .unwrap_or("127.0.0.1");
    let port = rcon
        .get("port")
        .and_then(Value::as_u64)
        .ok_or_else(|| "rcon.port is required".to_string())?;
    let password_file = rcon
        .get("passwordFile")
        .and_then(Value::as_str)
        .ok_or_else(|| "rcon.passwordFile is required".to_string())?;
    let password = fs::read_to_string(password_file)
        .map_err(|error| format!("read rcon password file: {error}"))?;
    let password = password.trim_end();
    if password.is_empty() {
        return Err("rcon password file is empty".to_string());
    }
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
#[path = "rcon_tests.rs"]
mod tests;
