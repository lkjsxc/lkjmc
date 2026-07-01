use std::{env, fs};

use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Config {
    pub application_id: Option<String>,
    pub public_key: Option<String>,
    pub register_commands: bool,
    pub interaction_bind: Option<String>,
    pub discord_token_file: Option<String>,
    pub discord_token_env: Option<String>,
    pub daemon_http_url: String,
    pub daemon_token_file: Option<String>,
    pub daemon_token_env: Option<String>,
    pub guild_allowlist: Vec<String>,
    pub channel_allowlist: Vec<String>,
    pub role_mappings: Vec<RoleMapping>,
    pub audit_actor: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleMapping {
    pub discord_role_id: String,
    pub lkjmc_role: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|error| format!("read config: {error}"))?;
        let value: Value =
            serde_json::from_str(&text).map_err(|error| format!("parse config: {error}"))?;
        Ok(Self {
            application_id: string(&value, "applicationId"),
            public_key: string(&value, "publicKey"),
            register_commands: bool_value(&value, "registerCommands"),
            interaction_bind: string(&value, "interactionBind"),
            discord_token_file: string(&value, "discordTokenFile"),
            discord_token_env: string(&value, "discordTokenEnv"),
            daemon_http_url: string(&value, "daemonHttpUrl").unwrap_or_default(),
            daemon_token_file: string(&value, "daemonTokenFile"),
            daemon_token_env: string(&value, "daemonTokenEnv"),
            guild_allowlist: strings(&value, "guildAllowlist"),
            channel_allowlist: strings(&value, "channelAllowlist"),
            role_mappings: role_mappings(&value),
            audit_actor: string(&value, "auditActor").unwrap_or_else(|| "discord-bot".into()),
        })
    }

    pub fn validate(self) -> Result<Self, String> {
        self.discord_secret()?;
        self.daemon_secret()?;
        if !self.daemon_http_url.starts_with("http://127.0.0.1")
            && !self.daemon_http_url.starts_with("http://localhost")
        {
            return Err("daemonHttpUrl must be loopback HTTP".into());
        }
        if self.guild_allowlist.is_empty() {
            return Err("guildAllowlist is empty".into());
        }
        if self.register_commands && self.application_id.as_deref().unwrap_or("").is_empty() {
            return Err("applicationId is required for command registration".into());
        }
        if self.interaction_bind.is_some() && self.public_key.as_deref().unwrap_or("").is_empty() {
            return Err("publicKey is required for interaction handling".into());
        }
        for mapping in &self.role_mappings {
            if mapping.discord_role_id.is_empty() || mapping.lkjmc_role.is_empty() {
                return Err("role mapping is incomplete".into());
            }
        }
        Ok(self)
    }

    pub fn discord_secret(&self) -> Result<String, String> {
        secret(&self.discord_token_file, &self.discord_token_env)
    }

    pub fn daemon_secret(&self) -> Result<String, String> {
        secret(&self.daemon_token_file, &self.daemon_token_env)
    }
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

fn bool_value(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
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
    fn missing_secret_is_redacted() {
        assert_eq!(
            secret(&None, &None).err().as_deref(),
            Some("secret source is missing")
        );
    }
}
