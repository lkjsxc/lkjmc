use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::config::{Config, RoleMapping};

pub struct Principal {
    pub user_id: String,
    pub roles: Vec<String>,
}

pub enum CommandPlan {
    Daemon { command: &'static str, body: Value },
    Immediate(String),
}

pub fn command_payload() -> Value {
    json!([{"name":"lkjmc","description":"lkjmc network controls","options":[
        sub("status", "Show daemon status", vec![]),
        sub("servers", "List managed servers", vec![]),
        sub("wake", "Request wake-and-join", vec![string("server", "Server id", true)]),
        sub("announce", "Publish an announcement", vec![string("message", "Message", true), string("server", "Server id", false)]),
        sub("reports", "List open reports", vec![]),
        sub("link", "Complete account linking", vec![string("code", "Link code from Minecraft", true)]),
        sub("unlink", "Remove account linking", vec![]),
        group("admin", "Admin operations", vec![
            sub("inspect", "Inspect grants", vec![user("user", "Discord user", true)]),
            sub("grant", "Grant role", vec![user("user", "Discord user", true), string("role", "lkjmc role", true), string("reason", "Reason", true)]),
            sub("revoke", "Revoke role", vec![user("user", "Discord user", true), string("role", "lkjmc role", true), string("reason", "Reason", true)]),
        ]),
        group("audit", "Audit operations", vec![sub("tail", "Show recent audit", vec![])]),
    ]}])
}

pub fn plan(
    path: &[String],
    options: &BTreeMap<String, String>,
    principal: &Principal,
    config: &Config,
) -> Result<CommandPlan, String> {
    let mut body = principal_body(principal, &config.role_mappings);
    match path
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["status"] => Ok(daemon("status", body)),
        ["servers"] => Ok(daemon("instance.list", body)),
        ["reports"] => {
            body["limit"] = json!(20);
            Ok(daemon("player.report.list", body))
        }
        ["audit", "tail"] => {
            body["lines"] = json!(20);
            Ok(daemon("admin.audit.tail", body))
        }
        ["announce"] => {
            body["actorName"] = json!(config.audit_actor);
            body["serverId"] = json!(options
                .get("server")
                .map(String::as_str)
                .unwrap_or("global"));
            body["message"] = json!(required(options, "message")?);
            Ok(daemon("announcement.create", body))
        }
        ["admin", "inspect"] => {
            subject(options, &mut body).map(|()| daemon("admin.principal.inspect", body))
        }
        ["admin", "grant"] => grant("admin.grant.create", options, body),
        ["admin", "revoke"] => grant("admin.grant.revoke", options, body),
        ["link"] => {
            body["code"] = json!(required(options, "code")?);
            Ok(daemon("discord.link.complete", body))
        }
        ["unlink"] => Ok(daemon("discord.link.remove", body)),
        ["wake"] => Ok(CommandPlan::Immediate(
            "Minecraft account linking is required before this Discord action.".into(),
        )),
        _ => Err("unsupported lkjmc Discord command".into()),
    }
}

pub fn format_daemon_response(value: &Value) -> String {
    if value.get("ok").and_then(Value::as_bool) == Some(true) {
        let body = value.get("body").cloned().unwrap_or_else(|| json!({}));
        return crate::formatting::format_body(&body)
            .unwrap_or_else(|| format!("ok {}", compact(&body)));
    }
    let code = value
        .pointer("/error/code")
        .and_then(Value::as_str)
        .unwrap_or("error");
    let message = value
        .pointer("/error/message")
        .and_then(Value::as_str)
        .unwrap_or("failed");
    format!("{code}: {}", redact(message))
}

fn principal_body(principal: &Principal, mappings: &[RoleMapping]) -> Value {
    let mapped = mappings
        .iter()
        .filter(|mapping| principal.roles.contains(&mapping.discord_role_id))
        .map(|mapping| mapping.lkjmc_role.clone())
        .collect::<Vec<_>>();
    json!({
        "principalKind": "discord-user",
        "principalId": principal.user_id,
        "discordRoles": principal.roles,
        "mappedRoles": mapped
    })
}

fn grant(
    command: &'static str,
    options: &BTreeMap<String, String>,
    mut body: Value,
) -> Result<CommandPlan, String> {
    subject(options, &mut body)?;
    body["roleId"] = json!(required(options, "role")?);
    body["reason"] = json!(required(options, "reason")?);
    Ok(daemon(command, body))
}

fn subject(options: &BTreeMap<String, String>, body: &mut Value) -> Result<(), String> {
    body["subjectKind"] = json!("discord-user");
    body["subjectId"] = json!(required(options, "user")?);
    Ok(())
}

fn daemon(command: &'static str, body: Value) -> CommandPlan {
    CommandPlan::Daemon { command, body }
}

fn required(options: &BTreeMap<String, String>, key: &str) -> Result<String, String> {
    options
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing option: {key}"))
}

fn sub(name: &str, description: &str, options: Vec<Value>) -> Value {
    json!({"type":1,"name":name,"description":description,"options":options})
}

fn group(name: &str, description: &str, options: Vec<Value>) -> Value {
    json!({"type":2,"name":name,"description":description,"options":options})
}

fn string(name: &str, description: &str, required: bool) -> Value {
    json!({"type":3,"name":name,"description":description,"required":required})
}

fn user(name: &str, description: &str, required: bool) -> Value {
    json!({"type":6,"name":name,"description":description,"required":required})
}

fn compact(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".into())
}

fn redact(value: &str) -> String {
    value.replace("Bearer ", "Bearer <redacted>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_to_daemon_with_discord_principal() -> Result<(), String> {
        let config = Config {
            application_id: None,
            public_key: None,
            register_commands: false,
            interaction_bind: None,
            discord_token_file: None,
            discord_token_env: Some("NOPE".into()),
            daemon_http_url: "http://127.0.0.1:1".into(),
            daemon_token_file: None,
            daemon_token_env: Some("NOPE".into()),
            guild_allowlist: vec!["g".into()],
            channel_allowlist: vec![],
            role_mappings: vec![],
            audit_actor: "discord".into(),
        };
        let principal = Principal {
            user_id: "u".into(),
            roles: vec![],
        };
        let planned = plan(&["status".into()], &BTreeMap::new(), &principal, &config)?;
        let CommandPlan::Daemon { command, body } = planned else {
            return Err("daemon plan expected".into());
        };
        assert_eq!(command, "status");
        assert_eq!(body["principalKind"], "discord-user");
        Ok(())
    }
}
