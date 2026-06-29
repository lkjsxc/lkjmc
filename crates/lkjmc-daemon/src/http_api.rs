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
    let request = read_request(&mut stream)?;
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

fn read_request(stream: &mut TcpStream) -> Result<String, String> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        let size = stream
            .read(&mut chunk)
            .map_err(|error| format!("read http: {error}"))?;
        if size == 0 {
            return Err("read http: connection closed".to_string());
        }
        buffer.extend_from_slice(&chunk[..size]);
        let Some(header_end) = header_end(&buffer) else {
            continue;
        };
        let headers = String::from_utf8_lossy(&buffer[..header_end]);
        let body_len = content_length(&headers);
        if buffer.len() >= header_end + body_len {
            return Ok(String::from_utf8_lossy(&buffer).to_string());
        }
    }
}

fn header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn content_length(headers: &str) -> usize {
    headers
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(str::trim)
                .map(str::to_string)
        })
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

fn authorized(request: &str, token: Option<&str>) -> bool {
    let Some(token) = token else {
        return false;
    };
    request
        .lines()
        .filter_map(authorization_bearer)
        .any(|value| value == token)
}

fn authorization_bearer(line: &str) -> Option<&str> {
    let (name, value) = line.split_once(':')?;
    if !name.trim().eq_ignore_ascii_case("authorization") {
        return None;
    }
    let mut parts = value.trim().splitn(2, char::is_whitespace);
    let scheme = parts.next()?;
    let token = parts.next()?.trim();
    if scheme.eq_ignore_ascii_case("bearer") {
        Some(token)
    } else {
        None
    }
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

#[cfg(test)]
mod tests {
    use super::authorized;

    #[test]
    fn authorization_preserves_token_case() {
        let request = "POST / HTTP/1.1\r\nAuthorization: Bearer AbC123+/=\r\n\r\n{}";
        assert!(authorized(request, Some("AbC123+/=")));
        assert!(!authorized(request, Some("abc123+/=")));
    }

    #[test]
    fn authorization_accepts_case_insensitive_scheme_and_name() {
        let request = "POST / HTTP/1.1\r\naUtHoRiZaTiOn: bEaReR MixedCase\r\n\r\n{}";
        assert!(authorized(request, Some("MixedCase")));
    }

    #[test]
    fn authorization_rejects_missing_or_wrong_token() {
        assert!(!authorized("POST / HTTP/1.1\r\n\r\n{}", Some("token")));
        assert!(!authorized("Authorization: Bearer token\r\n", None));
        assert!(!authorized("Authorization: Basic token\r\n", Some("token")));
    }
}
