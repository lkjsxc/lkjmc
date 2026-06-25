use crate::args::value_after;
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnouncementCommand {
    Send { server_id: String, message: String },
}

pub fn parse(values: &[String]) -> Result<AnnouncementCommand, CliError> {
    match values {
        [sub, rest @ ..] if sub == "send" => send(rest),
        _ => Err(CliError::message(usage())),
    }
}

fn send(values: &[String]) -> Result<AnnouncementCommand, CliError> {
    if values.len() != 4 {
        return Err(CliError::message(usage()));
    }
    let mut server_id = String::new();
    let mut message = String::new();
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--server" => server_id = value_after(values, index, "--server")?,
            "--message" => message = value_after(values, index, "--message")?,
            _ => return Err(CliError::message(usage())),
        }
        index += 2;
    }
    Ok(AnnouncementCommand::Send { server_id, message })
}

fn usage() -> &'static str {
    "usage: lkjmc announcement send --server SERVER --message MESSAGE"
}
