use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;

use crate::command_shapes::matches_type;
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
#[serde(untagged)]
pub enum RequestContract {
    Fields(FieldsRequest),
    HandlerDefined(HandlerDefinedRequest),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FieldsRequest {
    pub fields: BTreeMap<String, FieldContract>,
    #[serde(default, rename = "requiredAnyOf")]
    pub required_any_of: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HandlerDefinedRequest {
    pub body: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FieldContract {
    pub required: bool,
    #[serde(rename = "type")]
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ValueType {
    Array,
    Boolean,
    EmptyObject,
    Integer,
    Number,
    RconConfig,
    ShopMetadata,
    String,
    WorldLocation,
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
    let RequestContract::Fields(request) = &contract.request else {
        if let RequestContract::HandlerDefined(request) = &contract.request {
            if request.body != "handler-defined" {
                return Err("invalid handler-defined request".to_string());
            }
        }
        return if object.is_empty() {
            Ok(())
        } else {
            Err("command accepts no body members".to_string())
        };
    };
    for (name, field) in &request.fields {
        if field.required && !object.contains_key(name) {
            return Err(format!("missing required body member: {name}"));
        }
    }
    for group in &request.required_any_of {
        if !group.iter().any(|name| object.contains_key(name)) {
            return Err(format!("missing one of body members: {}", group.join(", ")));
        }
    }
    for (name, value) in object {
        let field = request
            .fields
            .get(name)
            .ok_or_else(|| format!("unknown body member: {name}"))?;
        if !matches_type(value, &field.value_type) {
            return Err(format!("wrong type for body member: {name}"));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "command_registry_tests.rs"]
mod tests;
