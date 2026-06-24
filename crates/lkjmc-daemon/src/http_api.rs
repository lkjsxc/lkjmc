use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
use lkjmc_core::id::CommandId;

use crate::api;
use crate::app::AppState;

pub fn serve(addr: &str, state: AppState, token: Option<String>) -> Result<(), String> {
    let listener = TcpListener::bind(addr).map_err(|error| format!("bind http {addr}: {error}"))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = state.clone();
                let token = token.clone();
                thread::spawn(move || {
                    if let Err(error) = handle(stream, state, token) {
                        eprintln!("http request failed: {error}");
                    }
                });
            }
            Err(error) => eprintln!("http accept failed: {error}"),
        }
    }
    Ok(())
}

fn handle(mut stream: TcpStream, state: AppState, token: Option<String>) -> Result<(), String> {
    let mut buffer = vec![0_u8; 16 * 1024];
    let size = stream
        .read(&mut buffer)
        .map_err(|error| format!("read http: {error}"))?;
    let request = String::from_utf8_lossy(&buffer[..size]);
    if !authorized(&request, token.as_deref()) {
        return write_http(&mut stream, 403, "{\"ok\":false}");
    }
    let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();
    let response = match serde_json::from_str::<CommandEnvelope>(body) {
        Ok(envelope) => api::dispatch(&state, envelope),
        Err(error) => api::error(
            invalid_request(),
            "request.invalid_json",
            error.to_string(),
            false,
        ),
    };
    let encoded =
        serde_json::to_string(&response).map_err(|error| format!("encode http: {error}"))?;
    write_http(&mut stream, 200, &encoded)
}

fn authorized(request: &str, token: Option<&str>) -> bool {
    let Some(token) = token else {
        return false;
    };
    let expected = format!("authorization: bearer {token}");
    request
        .lines()
        .any(|line| line.to_ascii_lowercase() == expected)
}

fn write_http(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let response = format!(
        "HTTP/1.1 {status} OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("write http: {error}"))
}

fn invalid_request() -> CommandEnvelope {
    CommandEnvelope {
        request_id: CommandId::internal("http-decode-error"),
        actor: Actor {
            kind: ActorKind::Daemon,
            name: "lkjmc-daemon".to_string(),
        },
        command: "decode-error".to_string(),
        body: serde_json::json!({}),
    }
}
