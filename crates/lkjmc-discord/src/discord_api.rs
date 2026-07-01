use serde_json::{json, Value};

use crate::config::Config;

pub fn register(config: &Config, commands: &Value) -> Result<(), String> {
    let token = config.discord_secret()?;
    let app = config
        .application_id
        .as_deref()
        .ok_or_else(|| "applicationId is required".to_string())?;
    for guild in &config.guild_allowlist {
        let url = format!("https://discord.com/api/v10/applications/{app}/guilds/{guild}/commands");
        ureq::put(&url)
            .set("authorization", &format!("Bot {token}"))
            .set("content-type", "application/json")
            .send_json(commands.clone())
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn followup(application_id: &str, token: &str, content: &str) -> Result<(), String> {
    let url = format!("https://discord.com/api/v10/webhooks/{application_id}/{token}");
    let safe = content.replace("Bearer ", "Bearer <redacted>");
    ureq::post(&url)
        .set("content-type", "application/json")
        .send_json(json!({
            "content": safe.chars().take(1800).collect::<String>(),
            "flags": 64
        }))
        .map_err(|error| error.to_string())?;
    Ok(())
}
