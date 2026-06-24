use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope, CommandResponse};
use lkjmc_core::id::CommandId;
use serde_json::Value;
use uuid::Uuid;

use crate::error::CliError;

pub fn call(socket: &str, command: &str, body: Value) -> Result<CommandResponse, CliError> {
    let request = CommandEnvelope {
        request_id: CommandId::parse("request id", Uuid::new_v4().to_string())?,
        actor: Actor {
            kind: ActorKind::Cli,
            name: "local-shell".to_string(),
        },
        command: command.to_string(),
        body,
    };
    let mut stream = UnixStream::connect(socket)?;
    let encoded = serde_json::to_string(&request)?;
    stream.write_all(format!("{encoded}\n").as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut line = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut line)?;
    let response = serde_json::from_str::<CommandResponse>(&line)?;
    Ok(response)
}
