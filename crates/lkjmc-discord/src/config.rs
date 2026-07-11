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
    pub guild_allowlist: Vec<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|error| format!("read config: {error}"))?;
        let value: Value =
            serde_json::from_str(&text).map_err(|error| format!("parse config: {error}"))?;
        Ok(Self {
            application_id: string(&value, "applicationId"),
            public_key: string(&value, "publicKey"),
            register_commands: boolean(&value, "registerCommands"),
            interaction_bind: string(&value, "interactionBind"),
            discord_token_file: string(&value, "discordTokenFile"),
            discord_token_env: string(&value, "discordTokenEnv"),
            guild_allowlist: strings(&value, "guildAllowlist"),
        })
    }

    pub fn validate(self) -> Result<Self, String> {
        self.discord_secret()?;
        if self.interaction_bind.is_some() && self.public_key.as_deref().unwrap_or("").is_empty() {
            return Err("publicKey is required for interaction handling".into());
        }
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
}
