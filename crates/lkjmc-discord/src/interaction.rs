use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::thread;

use crate::commands::{self, CommandPlan, Principal};
use crate::config::Config;

pub(crate) fn handle_interaction(
    config: &Config,
    headers: &HashMap<String, String>,
    body: &str,
) -> (u16, Value) {
    if let Err(error) = crate::signature::verify(config, headers, body) {
        return (401, response(&format!("signature denied: {error}")));
    }
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return (400, response("invalid interaction JSON"));
    };
    if value.get("type").and_then(Value::as_i64) == Some(1) {
        return (200, json!({"type": 1}));
    }
    match plan_interaction(config, &value) {
        Ok(CommandPlan::Immediate(message)) => (200, response(&message)),
        Ok(plan @ CommandPlan::Daemon { .. }) => defer_and_followup(config, &value, plan),
        Err(error) => (200, response(&error)),
    }
}

fn defer_and_followup(config: &Config, value: &Value, plan: CommandPlan) -> (u16, Value) {
    let app = value
        .get("application_id")
        .and_then(Value::as_str)
        .or(config.application_id.as_deref());
    let token = value.get("token").and_then(Value::as_str);
    let (Some(app), Some(token)) = (app, token) else {
        return (200, response("interaction token is missing"));
    };
    let config = config.clone();
    let app = app.to_string();
    let token = token.to_string();
    thread::spawn(move || {
        let message = execute(&config, plan).unwrap_or_else(|error| error);
        if let Err(error) = crate::discord_api::followup(&app, &token, &message) {
            eprintln!("discord follow-up failed: {error}");
        }
    });
    (200, json!({"type": 5, "data": {"flags": 64}}))
}

fn plan_interaction(config: &Config, value: &Value) -> Result<CommandPlan, String> {
    let guild = value
        .get("guild_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !config.guild_allowlist.iter().any(|item| item == guild) {
        return Err("guild is not allowed".into());
    }
    let channel = value
        .get("channel_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !config.channel_allowlist.is_empty()
        && !config.channel_allowlist.iter().any(|v| v == channel)
    {
        return Err("channel is not allowed".into());
    }
    let data = value
        .get("data")
        .ok_or_else(|| "missing command data".to_string())?;
    if data.get("name").and_then(Value::as_str) != Some("lkjmc") {
        return Err("unknown command root".into());
    }
    let (path, options) = command_path(data)?;
    commands::plan(&path, &options, &principal(value), config)
}

fn execute(config: &Config, plan: CommandPlan) -> Result<String, String> {
    match plan {
        CommandPlan::Immediate(message) => Ok(message),
        CommandPlan::Daemon { command, body } => crate::daemon::send(config, command, body)
            .map(|value| commands::format_daemon_response(&value)),
    }
}

fn principal(value: &Value) -> Principal {
    let member = value.get("member").unwrap_or(&Value::Null);
    let user = member
        .get("user")
        .or_else(|| value.get("user"))
        .unwrap_or(&Value::Null);
    let roles = member
        .get("roles")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    Principal {
        user_id: user
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        roles,
    }
}

fn command_path(data: &Value) -> Result<(Vec<String>, BTreeMap<String, String>), String> {
    let mut path = Vec::new();
    let mut options = BTreeMap::new();
    let mut cursor = data
        .get("options")
        .and_then(Value::as_array)
        .and_then(|items| items.first());
    while let Some(item) = cursor {
        let kind = item.get("type").and_then(Value::as_i64).unwrap_or(0);
        if kind != 1 && kind != 2 {
            break;
        }
        path.push(
            item.get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        );
        cursor = item
            .get("options")
            .and_then(Value::as_array)
            .and_then(|items| items.first());
        collect_values(item, &mut options);
    }
    if path.is_empty() {
        return Err("missing subcommand".into());
    }
    Ok((path, options))
}

fn collect_values(item: &Value, options: &mut BTreeMap<String, String>) {
    if let Some(items) = item.get("options").and_then(Value::as_array) {
        for option in items {
            let kind = option.get("type").and_then(Value::as_i64).unwrap_or(0);
            if kind == 1 || kind == 2 {
                continue;
            }
            if let Some(name) = option.get("name").and_then(Value::as_str) {
                let value = option
                    .get("value")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                options.insert(name.to_string(), value.to_string());
            }
        }
    }
}

fn response(content: &str) -> Value {
    let safe = content.replace("Bearer ", "Bearer <redacted>");
    json!({"type": 4, "data": {"content": safe.chars().take(1800).collect::<String>(), "flags": 64}})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_nested_command_path_and_options() {
        let data = json!({"options":[{"type":2,"name":"admin","options":[{"type":1,"name":"grant","options":[{"type":3,"name":"role","value":"owner"}]}]}]});
        let (path, options) = command_path(&data).unwrap();
        assert_eq!(path, vec!["admin", "grant"]);
        assert_eq!(options.get("role").map(String::as_str), Some("owner"));
    }
}
