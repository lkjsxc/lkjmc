use lkjmc_core::command::CommandEnvelope;
use lkjmc_core::random_teleport::RandomTeleportPolicy;
use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::instance_helpers::{body_string, store, with_connection};

type Response = lkjmc_core::command::CommandResponse;

pub fn snapshot(state: &AppState, request: CommandEnvelope) -> Response {
    with_connection(state, request, |_state, request, client| {
        let player_uuid = parse_uuid(&request, "playerUuid")?;
        let server_id = body_string(&request.body, "serverId")?;
        let hud = store(lkjmc_store::player_settings::hud_enabled(
            client,
            player_uuid,
        ))?
        .unwrap_or(true);
        let playtime = store(lkjmc_store::player_session::playtime_seconds(
            client,
            player_uuid,
        ))?;
        let balance = store(lkjmc_store::points::balance(client, player_uuid))?;
        let server_players = store(lkjmc_store::player_session::active_count_for_server(
            client, &server_id,
        ))?;
        let network_online = store(lkjmc_store::player_session::active_count(client))?;
        let daily_available = !store(lkjmc_store::daily::claimed_today(client, player_uuid))?;
        let policy = RandomTeleportPolicy::defaults();
        let rtp_cooldown = store(lkjmc_store::random_teleport::cooldown_remaining(
            client,
            player_uuid,
            &server_id,
            policy.cooldown_seconds,
        ))?;
        Ok(api::ok(
            request,
            json!({
                "hudEnabled": hud,
                "playtimeSeconds": playtime,
                "balance": balance,
                "serverId": server_id,
                "serverPlayerCount": server_players,
                "networkOnlineCount": network_online,
                "dailyAvailable": daily_available,
                "randomTeleportCooldownSeconds": rtp_cooldown
            }),
        ))
    })
}

fn parse_uuid(request: &CommandEnvelope, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&request.body, field)?).map_err(|error| error.to_string())
}
