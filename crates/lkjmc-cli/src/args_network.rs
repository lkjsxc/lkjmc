use crate::args::{value_after, CliCommand};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkCommand {
    Diagnose(NetworkDiagnoseOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkDiagnoseOptions {
    pub host: String,
    pub port: u16,
    pub expected_address: Option<String>,
    pub direct_address: Option<String>,
}

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [sub, host, rest @ ..] if sub == "diagnose" => Ok(CliCommand::Network(
            NetworkCommand::Diagnose(options(host.clone(), rest)?),
        )),
        _ => Err(CliError::message(
            "usage: lkjmc network diagnose HOST [--port PORT]",
        )),
    }
}

fn options(host: String, values: &[String]) -> Result<NetworkDiagnoseOptions, CliError> {
    let mut options = NetworkDiagnoseOptions {
        host,
        port: 25565,
        expected_address: None,
        direct_address: None,
    };
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--port" => {
                options.port = parse_port(&value_after(values, index, "--port")?)?;
                index += 2;
            }
            "--expect-address" => {
                options.expected_address = Some(value_after(values, index, "--expect-address")?);
                index += 2;
            }
            "--direct-address" => {
                options.direct_address = Some(value_after(values, index, "--direct-address")?);
                index += 2;
            }
            other => return Err(CliError::message(format!("unknown network flag: {other}"))),
        }
    }
    Ok(options)
}

fn parse_port(value: &str) -> Result<u16, CliError> {
    value
        .parse::<u16>()
        .map_err(|error| CliError::message(format!("invalid port: {error}")))
}
