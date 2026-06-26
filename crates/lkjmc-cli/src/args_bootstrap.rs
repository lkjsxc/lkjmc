use crate::args::{value_after, CliCommand};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapCommand {
    Plan(BootstrapOptions),
    Apply(BootstrapOptions),
    Status,
    Doctor { host: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapOptions {
    pub profile: String,
    pub accept_minecraft_eula: bool,
    pub bedrock: Option<String>,
    pub java_bind_host: Option<String>,
    pub java_port: Option<u16>,
    pub java_public_host: Option<String>,
    pub bedrock_port: Option<u16>,
}

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [sub, rest @ ..] if sub == "plan" => Ok(CliCommand::Bootstrap(parse_plan(rest)?)),
        [sub, rest @ ..] if sub == "apply" => Ok(CliCommand::Bootstrap(parse_apply(rest)?)),
        [sub] if sub == "status" => Ok(CliCommand::Bootstrap(BootstrapCommand::Status)),
        [sub, rest @ ..] if sub == "doctor" => Ok(CliCommand::Bootstrap(parse_doctor(rest)?)),
        _ => Err(CliError::message(
            "usage: lkjmc bootstrap plan|apply|status|doctor",
        )),
    }
}

fn parse_doctor(values: &[String]) -> Result<BootstrapCommand, CliError> {
    let mut host = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--host" => {
                host = Some(value_after(values, index, "--host")?);
                index += 2;
            }
            other => {
                return Err(CliError::message(format!(
                    "unknown bootstrap doctor flag: {other}"
                )))
            }
        }
    }
    Ok(BootstrapCommand::Doctor { host })
}

fn parse_plan(values: &[String]) -> Result<BootstrapCommand, CliError> {
    Ok(BootstrapCommand::Plan(options(values, false)?))
}

fn parse_apply(values: &[String]) -> Result<BootstrapCommand, CliError> {
    options(values, true).map(BootstrapCommand::Apply)
}

fn options(values: &[String], allow_accept: bool) -> Result<BootstrapOptions, CliError> {
    let mut options = BootstrapOptions {
        profile: "playable".to_string(),
        accept_minecraft_eula: false,
        bedrock: None,
        java_bind_host: None,
        java_port: None,
        java_public_host: None,
        bedrock_port: None,
    };
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--profile" => {
                options.profile = value_after(values, index, "--profile")?;
                index += 2;
            }
            "--bedrock" => {
                options.bedrock = Some(value_after(values, index, "--bedrock")?);
                index += 2;
            }
            "--java-bind-host" => {
                options.java_bind_host = Some(value_after(values, index, "--java-bind-host")?);
                index += 2;
            }
            "--java-port" => {
                options.java_port = Some(parse_port(&value_after(values, index, "--java-port")?)?);
                index += 2;
            }
            "--java-public-host" => {
                options.java_public_host = Some(value_after(values, index, "--java-public-host")?);
                index += 2;
            }
            "--bedrock-port" => {
                options.bedrock_port =
                    Some(parse_port(&value_after(values, index, "--bedrock-port")?)?);
                index += 2;
            }
            "--accept-minecraft-eula" if allow_accept => {
                options.accept_minecraft_eula = true;
                index += 1;
            }
            other => {
                return Err(CliError::message(format!(
                    "unknown bootstrap flag: {other}"
                )))
            }
        }
    }
    Ok(options)
}

fn parse_port(value: &str) -> Result<u16, CliError> {
    value
        .parse::<u16>()
        .map_err(|error| CliError::message(format!("invalid port: {error}")))
}
