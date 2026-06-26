use crate::args::{value_after, CliCommand};
use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapCommand {
    Plan {
        profile: String,
        bedrock: Option<String>,
    },
    Apply {
        profile: String,
        accept_minecraft_eula: bool,
        bedrock: Option<String>,
    },
    Status,
    Doctor,
}

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [sub, rest @ ..] if sub == "plan" => Ok(CliCommand::Bootstrap(parse_plan(rest)?)),
        [sub, rest @ ..] if sub == "apply" => Ok(CliCommand::Bootstrap(parse_apply(rest)?)),
        [sub] if sub == "status" => Ok(CliCommand::Bootstrap(BootstrapCommand::Status)),
        [sub] if sub == "doctor" => Ok(CliCommand::Bootstrap(BootstrapCommand::Doctor)),
        _ => Err(CliError::message(
            "usage: lkjmc bootstrap plan|apply|status|doctor",
        )),
    }
}

fn parse_plan(values: &[String]) -> Result<BootstrapCommand, CliError> {
    let (profile, bedrock, _) = options(values)?;
    Ok(BootstrapCommand::Plan { profile, bedrock })
}

fn parse_apply(values: &[String]) -> Result<BootstrapCommand, CliError> {
    let (profile, bedrock, accept_minecraft_eula) = options(values)?;
    Ok(BootstrapCommand::Apply {
        profile,
        accept_minecraft_eula,
        bedrock,
    })
}

fn options(values: &[String]) -> Result<(String, Option<String>, bool), CliError> {
    let mut profile = "playable".to_string();
    let mut bedrock = None;
    let mut accept = false;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--profile" => {
                profile = value_after(values, index, "--profile")?;
                index += 2;
            }
            "--bedrock" => {
                bedrock = Some(value_after(values, index, "--bedrock")?);
                index += 2;
            }
            "--accept-minecraft-eula" => {
                accept = true;
                index += 1;
            }
            other => {
                return Err(CliError::message(format!(
                    "unknown bootstrap flag: {other}"
                )))
            }
        }
    }
    Ok((profile, bedrock, accept))
}
