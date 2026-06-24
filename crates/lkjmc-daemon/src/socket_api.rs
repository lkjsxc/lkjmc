use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;

use crate::api;
use crate::app::AppState;

pub fn serve(path: &str, state: AppState) -> Result<(), String> {
    if fs::metadata(path).is_ok() {
        fs::remove_file(path).map_err(|error| format!("remove socket {path}: {error}"))?;
    }
    let listener =
        UnixListener::bind(path).map_err(|error| format!("bind socket {path}: {error}"))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = state.clone();
                thread::spawn(move || {
                    if let Err(error) = handle(stream, state) {
                        eprintln!("socket request failed: {error}");
                    }
                });
            }
            Err(error) => eprintln!("socket accept failed: {error}"),
        }
    }
    Ok(())
}

fn handle(mut stream: UnixStream, state: AppState) -> Result<(), String> {
    let mut line = String::new();
    {
        let mut reader = BufReader::new(&mut stream);
        reader
            .read_line(&mut line)
            .map_err(|error| format!("read request: {error}"))?;
    }
    let response = match serde_json::from_str::<CommandEnvelope>(&line) {
        Ok(request) => api::dispatch(&state, request),
        Err(error) => decode_error(error.to_string()),
    };
    let encoded =
        serde_json::to_string(&response).map_err(|error| format!("encode response: {error}"))?;
    stream
        .write_all(format!("{encoded}\n").as_bytes())
        .map_err(|error| format!("write response: {error}"))?;
    Ok(())
}

fn decode_error(message: String) -> CommandResponse {
    let request = CommandEnvelope {
        request_id: CommandId::internal("decode-error"),
        actor: Actor {
            kind: ActorKind::Daemon,
            name: "lkjmc-daemon".to_string(),
        },
        command: "decode-error".to_string(),
        body: serde_json::json!({}),
    };
    api::error(request, "request.invalid_json", message, false)
}
