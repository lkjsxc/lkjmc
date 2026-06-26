use crate::error::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimCommand {
    List { instance: String },
    Delete { claim_id: String, yes: bool },
}

pub fn parse(values: &[String]) -> Result<ClaimCommand, CliError> {
    match values {
        [sub, flag, instance] if sub == "list" && flag == "--instance" => Ok(ClaimCommand::List {
            instance: instance.clone(),
        }),
        [sub, claim_id, rest @ ..] if sub == "delete" => Ok(ClaimCommand::Delete {
            claim_id: claim_id.clone(),
            yes: rest.iter().any(|value| value == "--yes"),
        }),
        _ => Err(CliError::message(usage())),
    }
}

fn usage() -> &'static str {
    "usage: lkjmc claim list --instance INSTANCE | claim delete CLAIM_ID --yes"
}
