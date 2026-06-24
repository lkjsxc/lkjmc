use lkjmc_core::command::CommandResponse;
use serde_json::{json, Value};

use crate::error::CliError;

pub fn response_body(response: CommandResponse) -> Result<Value, CliError> {
    if response.ok {
        match response.body {
            Some(body) => Ok(body),
            None => Ok(json!({})),
        }
    } else {
        let message = match response.error {
            Some(error) => format!("{}: {}", error.code, error.message),
            None => "unknown daemon error".to_string(),
        };
        Err(CliError::message(message))
    }
}

pub fn print_json(value: &Value) -> Result<(), CliError> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}
