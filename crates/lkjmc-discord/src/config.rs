use std::{env, fs};

use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Config {
    pub application_id: Option<String>,
    pub register_commands: bool,
    pub interaction_bind: Option<String>,
    pub discord_token_file: Option<String>,
    pub discord_token_env: Option<String>,
    pub guild_allowlist: Vec<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|error| format!("read config: {error}"))?;
        let value: Value =
            serde_json::from_str(&text).map_err(|error| format!("parse config: {error}"))?;
        Ok(Self {
            application_id: string(&value, "applicationId"),
            register_commands: boolean(&value, "registerCommands"),
            interaction_bind: string(&value, "interactionBind"),
            discord_token_file: string(&value, "discordTokenFile"),
            discord_token_env: string(&value, "discordTokenEnv"),
            guild_allowlist: strings(&value, "guildAllowlist"),
        })
    }

    pub fn validate(self) -> Result<Self, String> {
        if self.interaction_bind.is_some() {
            return Err("interaction listener is withdrawn; remove interactionBind".into());
        }
        self.discord_secret()?;
        if self.register_commands {
            self.validate_command_withdrawal()?;
        }
        Ok(self)
    }

    pub fn validate_command_withdrawal(&self) -> Result<(), String> {
        if self.application_id.as_deref().unwrap_or("").is_empty() {
            return Err("applicationId is required for command withdrawal".into());
        }
        if self.guild_allowlist.is_empty() {
            return Err("guildAllowlist is empty".into());
        }
        Ok(())
    }

    pub fn discord_secret(&self) -> Result<String, String> {
        secret(&self.discord_token_file, &self.discord_token_env)
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

fn boolean(value: &Value, key: &str) -> bool {
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

    #[test]
    fn interaction_bind_is_rejected_before_secret_lookup() {
        let config = Config {
            application_id: None,
            register_commands: false,
            interaction_bind: Some("127.0.0.1:8080".into()),
            discord_token_file: None,
            discord_token_env: None,
            guild_allowlist: Vec::new(),
        };
        assert_eq!(
            config.validate().err().as_deref(),
            Some("interaction listener is withdrawn; remove interactionBind")
        );
    }
}
