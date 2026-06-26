use serde_json::{json, Value};

use crate::args_claim::ClaimCommand;
use crate::commands::daemon_command;
use crate::error::CliError;

pub fn run(socket: &str, command: ClaimCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        ClaimCommand::List { instance } => list(socket, instance, json_output),
        ClaimCommand::Delete { claim_id, yes } => delete(socket, claim_id, yes, json_output),
    }
}

fn list(socket: &str, instance: String, json_output: bool) -> Result<(), CliError> {
    let response = crate::client::call(socket, "claim.snapshot", json!({"instanceId": instance}))?;
    let body = crate::format::response_body(response)?;
    if json_output {
        return crate::format::print_json(&body);
    }
    let count = body
        .get("chunks")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    println!("claims: {count}");
    for item in body
        .get("chunks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        println!("{}", claim_line(item));
    }
    Ok(())
}

fn delete(socket: &str, claim_id: String, yes: bool, json_output: bool) -> Result<(), CliError> {
    if !yes {
        return Err(CliError::message("claim delete requires --yes"));
    }
    daemon_command(
        socket,
        "claim.delete",
        json!({"claimId": claim_id, "operator": true}),
        json_output,
        "ok claim delete",
    )
}

fn claim_line(item: &Value) -> String {
    format!(
        "{} {}:{} ({},{}) owner={}",
        item.get("claimId").and_then(Value::as_str).unwrap_or("-"),
        item.get("instanceId")
            .and_then(Value::as_str)
            .unwrap_or("-"),
        item.get("worldName").and_then(Value::as_str).unwrap_or("-"),
        item.get("chunkX").and_then(Value::as_i64).unwrap_or(0),
        item.get("chunkZ").and_then(Value::as_i64).unwrap_or(0),
        item.get("ownerName").and_then(Value::as_str).unwrap_or("-")
    )
}
