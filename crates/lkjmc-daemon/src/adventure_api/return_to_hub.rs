use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::audit_helpers::audit;
use crate::instance_helpers::{body_string, store, with_client};

pub fn end(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    with_client(state, envelope, |_state, envelope, client| {
        let player_uuid = parse_uuid(&envelope, "playerUuid")?;
        let player_name = body_string(&envelope.body, "playerName")?;
        let instance_id = body_string(&envelope.body, "temporaryInstanceId")?;
        let temp = store(lkjmc_store::temporary::get_instance(client, &instance_id))?
            .ok_or_else(|| format!("temporary instance not found: {instance_id}"))?;
        if temp.owner_kind != "adventure" {
            return Err("current temporary instance is not an adventure".to_string());
        }
        let session = store(lkjmc_store::temporary::get_session_by_instance(
            client,
            &instance_id,
        ))?
        .ok_or_else(|| format!("adventure session not found: {instance_id}"))?;
        if session.adventure_kind != "end-expedition" {
            return Err("current adventure is not End Expedition".to_string());
        }
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &player_name,
        ))?;
        let changed = store(lkjmc_store::temporary::mark_participant_left(
            client,
            session.id,
            player_uuid,
        ))?;
        if changed == 0 {
            return Err("player is not an adventure participant".to_string());
        }
        let remaining = store(lkjmc_store::temporary::active_participant_count(
            client, session.id,
        ))?;
        let state = if remaining == 0 {
            store(lkjmc_store::temporary::update_session_state(
                client,
                session.id,
                "completed",
                None,
                None,
            ))?;
            "completed"
        } else {
            session.state.as_str()
        };
        audit(
            client,
            &envelope,
            "adventure.end.return",
            "adventure-session",
            &session.id.to_string(),
            "succeeded",
        )?;
        Ok(api::ok(
            envelope,
            json!({
                "sessionId": session.id.to_string(),
                "temporaryInstanceId": instance_id,
                "targetServer": "hub",
                "state": state,
                "remainingParticipants": remaining
            }),
        ))
    })
}

fn parse_uuid(
    envelope: &lkjmc_core::command::CommandEnvelope,
    field: &'static str,
) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(&envelope.body, field)?).map_err(|error| error.to_string())
}
