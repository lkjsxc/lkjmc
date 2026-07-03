use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::AppState;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::body_string;

const CODE_MINUTES: i32 = 10;
const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub fn begin(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match begin_inner(state, &request) {
        Ok(code) => api::ok(
            request,
            json!({"code": code, "expiresMinutes": CODE_MINUTES}),
        ),
        Err(error) => api::error(request, "link.begin_failed", error, false),
    }
}

pub fn remove_player(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match with_client(state, |client| {
        let player_uuid = player_uuid(&request)?;
        let removed = lkjmc_store::discord_links::remove_player(client, player_uuid)
            .map_err(|error| error.to_string())?;
        audit(
            client,
            &request,
            "player.link.remove",
            "player",
            &player_uuid.to_string(),
            "succeeded",
        )?;
        Ok(json!({"removed": removed}))
    }) {
        Ok(body) => api::ok(request, body),
        Err(error) => api::error(request, "link.remove_failed", error, false),
    }
}

pub fn complete(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match with_client(state, |client| {
        let code = body_string(&request.body, "code")?;
        let discord = discord_id(&request);
        let Some(row) = lkjmc_store::discord_links::complete(client, &discord, &hash(&code))
            .map_err(|error| error.to_string())?
        else {
            return Err("link code is invalid or expired".to_string());
        };
        audit(
            client,
            &request,
            "discord.link.complete",
            "discord-user",
            &discord,
            "succeeded",
        )?;
        Ok(json!({"playerUuid": row.player_uuid.to_string(), "playerName": row.player_name}))
    }) {
        Ok(body) => api::ok(request, body),
        Err(error) => api::error(request, "link.complete_failed", error, false),
    }
}

pub fn wake(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    let target = match body_string(&request.body, "targetInstanceId")
        .or_else(|_| body_string(&request.body, "server"))
    {
        Ok(value) => value,
        Err(error) => return api::error(request, "discord.wake_failed", error, false),
    };
    let discord = discord_id(&request);
    let linked = with_client(state, |client| {
        lkjmc_store::discord_links::find_by_discord(client, &discord)
            .map_err(|error| error.to_string())?
            .filter(|link| link.verified && !link.revoked)
            .ok_or_else(|| "Discord user is not linked to a Minecraft player".to_string())
    });
    let link = match linked {
        Ok(link) => link,
        Err(error) => return api::error(request, "discord.wake_failed", error, false),
    };
    crate::dispatch::dispatch(
        state,
        CommandEnvelope {
            command: "instance.wake.request".to_string(),
            body: json!({
                "playerUuid": link.minecraft_uuid.to_string(),
                "playerName": format!("discord:{}", discord),
                "targetInstanceId": target,
            }),
            ..request
        },
    )
}

pub fn remove_discord(state: &AppState, request: CommandEnvelope) -> CommandResponse {
    match with_client(state, |client| {
        let discord = discord_id(&request);
        let removed = lkjmc_store::discord_links::remove_discord(client, &discord)
            .map_err(|error| error.to_string())?;
        audit(
            client,
            &request,
            "discord.link.remove",
            "discord-user",
            &discord,
            "succeeded",
        )?;
        Ok(json!({"removed": removed}))
    }) {
        Ok(body) => api::ok(request, body),
        Err(error) => api::error(request, "link.remove_failed", error, false),
    }
}

fn begin_inner(state: &AppState, request: &CommandEnvelope) -> Result<String, String> {
    with_client(state, |client| {
        let player_uuid = player_uuid(request)?;
        let player_name = body_string(&request.body, "playerName")
            .or_else(|_| body_string(&request.body, "name"))?;
        let code = code();
        lkjmc_store::discord_links::begin(
            client,
            player_uuid,
            &player_name,
            &hash(&code),
            CODE_MINUTES,
        )
        .map_err(|error| error.to_string())?;
        audit(
            client,
            request,
            "player.link.begin",
            "player",
            &player_uuid.to_string(),
            "succeeded",
        )?;
        Ok(code)
    })
}

fn with_client<T>(
    state: &AppState,
    action: impl FnOnce(&mut postgres::Client) -> Result<T, String>,
) -> Result<T, String> {
    if state.database_url().is_none() {
        return Err("Database URL is not configured".to_string());
    }
    let mut client = state.database_connection()?;
    action(&mut client)
}

fn player_uuid(request: &CommandEnvelope) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, "playerUuid")?).map_err(|error| error.to_string())
}

fn discord_id(request: &CommandEnvelope) -> String {
    request
        .body
        .get("discordUserId")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&request.actor.name)
        .to_string()
}

fn hash(code: &str) -> String {
    format!("{:x}", Sha256::digest(code.as_bytes()))
}

fn code() -> String {
    let mut out = String::new();
    for byte in *Uuid::new_v4().as_bytes() {
        out.push(ALPHABET[(byte as usize) % ALPHABET.len()] as char);
        if out.len() == 8 {
            break;
        }
    }
    out
}
