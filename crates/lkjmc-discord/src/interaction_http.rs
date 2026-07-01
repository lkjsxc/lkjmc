use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::config::Config;

pub fn serve(addr: &str, config: Config) -> Result<(), String> {
    let listener = TcpListener::bind(addr).map_err(|error| format!("bind discord: {error}"))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_stream(stream, &config)?,
            Err(error) => eprintln!("discord interaction accept failed: {error}"),
        }
    }
    Ok(())
}

fn handle_stream(mut stream: TcpStream, config: &Config) -> Result<(), String> {
    let raw = read_request(&mut stream)?;
    let (headers, body) = split_request(&raw);
    let (status, value) = crate::interaction::handle_interaction(config, &headers, &body);
    let encoded = serde_json::to_string(&value).map_err(|error| error.to_string())?;
    let response = format!(
        "HTTP/1.1 {status} OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{encoded}",
        encoded.len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| error.to_string())
}

fn split_request(raw: &str) -> (HashMap<String, String>, String) {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let headers = head
        .lines()
        .skip(1)
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_ascii_lowercase(), value.trim().to_string()))
        })
        .collect();
    (headers, body.to_string())
}

fn read_request(stream: &mut TcpStream) -> Result<String, String> {
    let mut buffer = [0_u8; 65536];
    let size = stream
        .read(&mut buffer)
        .map_err(|error| error.to_string())?;
    Ok(String::from_utf8_lossy(&buffer[..size]).to_string())
}
