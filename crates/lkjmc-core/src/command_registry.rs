use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;

use crate::command_shards::SOURCES;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryShard {
    commands: Vec<CommandContract>,
    domain: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandContract {
    pub authorization: String,
    pub deadline: String,
    pub doc: String,
    pub effect: String,
    pub errors: String,
    pub handler: String,
    pub idempotency: String,
    pub identity: String,
    pub name: String,
    pub request: RequestContract,
    pub response: ResponseContract,
    pub summary: String,
    pub surfaces: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RequestContract {
    pub optional: Vec<String>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResponseContract {
    pub body: String,
    pub envelope: String,
}

static REGISTRY: OnceLock<Vec<CommandContract>> = OnceLock::new();

pub fn all() -> &'static [CommandContract] {
    REGISTRY.get_or_init(|| {
        let mut commands = Vec::new();
        for source in SOURCES {
            if let Ok(shard) = serde_json::from_str::<RegistryShard>(source) {
                if !shard.domain.is_empty() {
                    commands.extend(shard.commands);
                }
            }
        }
        commands.sort_by(|left, right| left.name.cmp(&right.name));
        commands
    })
}

pub fn contract_for(name: &str) -> Option<&'static CommandContract> {
    all().iter().find(|entry| entry.name == name)
}

pub fn validate_body(name: &str, body: &Value) -> Result<(), String> {
    let contract = contract_for(name).ok_or_else(|| "unknown command".to_string())?;
    let object = body
        .as_object()
        .ok_or_else(|| "body must be an object".to_string())?;
    for field in &contract.request.required {
        if !object.contains_key(field) {
            return Err(format!("missing required body member: {field}"));
        }
    }
    for field in object.keys() {
        let declared = contract
            .request
            .required
            .iter()
            .chain(&contract.request.optional);
        if !declared.into_iter().any(|allowed| allowed == field) {
            return Err(format!("unknown body member: {field}"));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "command_registry_tests.rs"]
mod tests;
