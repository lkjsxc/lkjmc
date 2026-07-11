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
