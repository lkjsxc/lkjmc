use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservabilityCommand {
    Events {
        request_id: Option<String>,
        operation_id: Option<String>,
        correlation_id: Option<String>,
        limit: i64,
    },
}

pub fn parse(values: &[String]) -> Result<ObservabilityCommand, CliError> {
    let Some(command) = values.first() else {
        return Err(usage());
    };
    if command != "events" {
        return Err(usage());
    }
    let mut request_id = None;
    let mut operation_id = None;
    let mut correlation_id = None;
    let mut limit = 100;
    let mut index = 1;
    while index < values.len() {
        let value = values.get(index + 1).cloned().ok_or_else(usage)?;
        match values[index].as_str() {
            "--request" => request_id = Some(value),
            "--operation" => operation_id = Some(value),
            "--correlation" => correlation_id = Some(value),
            "--limit" => limit = value.parse::<i64>().map_err(|_| usage())?,
            _ => return Err(usage()),
        }
        index += 2;
    }
    let filters = usize::from(request_id.is_some())
        + usize::from(operation_id.is_some())
        + usize::from(correlation_id.is_some());
    if filters > 1 || !(1..=500).contains(&limit) {
        return Err(usage());
    }
    Ok(ObservabilityCommand::Events {
        request_id,
        operation_id,
        correlation_id,
        limit,
    })
}

fn usage() -> CliError {
    CliError::message("usage: lkjmc observability events [--request ID|--operation UUID|--correlation UUID] [--limit N]")
}
