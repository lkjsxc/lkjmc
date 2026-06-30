#![forbid(unsafe_code)]

use std::env;
use std::fs;

use serde_json::{json, Value};
use uuid::Uuid;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let path = args.get(1).map(String::as_str).unwrap_or("discord.json");
    match Config::load(path).and_then(|config| config.validate()) {
        Ok(config) => {
            println!(
                "ok discord config guilds={} channels={} commands={}",
                config.guild_allowlist.len(),
                config.channel_allowlist.len(),
                slash_commands().len()
            );
            if args.iter().any(|arg| arg == "--daemon-status") {
                match daemon_status(&config) {
                    Ok(summary) => println!("ok daemon status {summary}"),
                    Err(error) => eprintln!("daemon status failed: {error}"),
                }
            }
        }
        Err(error) => {
            eprintln!("discord startup disabled: {error}");
            std::process::exit(1);
        }
    }
}

#[derive(Clone, Debug)]
struct Config {
    discord_token_file: Option<String>,
    discord_token_env: Option<String>,
    daemon_http_url: String,
    daemon_token_file: Option<String>,
    daemon_token_env: Option<String>,
    guild_allowlist: Vec<String>,
    channel_allowlist: Vec<String>,
    role_mappings: Vec<RoleMapping>,
    audit_actor: String,
}

#[derive(Clone, Debug)]
struct RoleMapping {
    discord_role_id: String,
    lkjmc_role: String,
}

impl Config {
    fn load(path: &str) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|error| format!("read config: {error}"))?;
        let value: Value =
            serde_json::from_str(&text).map_err(|error| format!("parse config: {error}"))?;
        Ok(Self {
            discord_token_file: string(&value, "discordTokenFile"),
            discord_token_env: string(&value, "discordTokenEnv"),
            daemon_http_url: string(&value, "daemonHttpUrl").unwrap_or_default(),
            daemon_token_file: string(&value, "daemonTokenFile"),
            daemon_token_env: string(&value, "daemonTokenEnv"),
            guild_allowlist: strings(&value, "guildAllowlist"),
            channel_allowlist: strings(&value, "channelAllowlist"),
            role_mappings: role_mappings(&value),
            audit_actor: string(&value, "auditActor").unwrap_or_else(|| "discord-bot".to_string()),
        })
    }

    fn validate(self) -> Result<Self, String> {
        if self.discord_secret().is_err() {
            return Err("discord token source is missing".to_string());
        }
        if self.daemon_secret().is_err() {
            return Err("daemon token source is missing".to_string());
        }
        if !self.daemon_http_url.starts_with("http://127.0.0.1")
            && !self.daemon_http_url.starts_with("http://localhost")
        {
            return Err("daemonHttpUrl must be loopback HTTP".to_string());
        }
        if self.guild_allowlist.is_empty() {
            return Err("guildAllowlist is empty".to_string());
        }
        for mapping in &self.role_mappings {
            if mapping.discord_role_id.is_empty() || mapping.lkjmc_role.is_empty() {
                return Err("role mapping is incomplete".to_string());
            }
        }
        Ok(self)
    }

    fn discord_secret(&self) -> Result<String, String> {
        secret(&self.discord_token_file, &self.discord_token_env)
    }
    fn daemon_secret(&self) -> Result<String, String> {
        secret(&self.daemon_token_file, &self.daemon_token_env)
    }
}

fn daemon_status(config: &Config) -> Result<String, String> {
    let token = config.daemon_secret()?;
    let body = json!({
        "requestId": Uuid::new_v4().to_string(),
        "actor": {"kind":"daemon", "name": config.audit_actor},
        "command": "status",
        "body": {"principalKind":"discord-user", "principalId":"startup-check"}
    });
    let response = ureq::post(&config.daemon_http_url)
        .set("authorization", &format!("Bearer {token}"))
        .send_json(body)
        .map_err(|error| error.to_string())?;
    let value: Value = response.into_json().map_err(|error| error.to_string())?;
    Ok(value
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        .to_string())
}

fn slash_commands() -> Vec<Value> {
    vec![
        json!({"name":"status", "description":"Show lkjmc daemon status"}),
        json!({"name":"servers", "description":"List managed servers"}),
        json!({"name":"wake", "description":"Request wake-and-join"}),
        json!({"name":"announce", "description":"Publish an audited announcement"}),
        json!({"name":"reports", "description":"List open reports"}),
        json!({"name":"link", "description":"Link Discord and Minecraft accounts"}),
    ]
}

fn secret(file: &Option<String>, env_key: &Option<String>) -> Result<String, String> {
    if let Some(path) = file {
        return fs::read_to_string(path)
            .map(|value| value.trim().to_string())
            .map_err(|_| "secret file is unreadable".to_string());
    }
    if let Some(key) = env_key {
        return env::var(key).map_err(|_| "secret environment variable is missing".to_string());
    }
    Err("secret source is missing".to_string())
}

fn string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn strings(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn role_mappings(value: &Value) -> Vec<RoleMapping> {
    value
        .get("roleMappings")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| RoleMapping {
                    discord_role_id: string(item, "discordRoleId").unwrap_or_default(),
                    lkjmc_role: string(item, "lkjmcRole").unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slash_commands_are_real_definitions() {
        assert!(slash_commands().len() >= 6);
    }
    #[test]
    fn missing_secret_is_redacted() {
        assert_eq!(
            secret(&None, &None).err().as_deref(),
            Some("secret source is missing")
        );
    }
}
