use serde_json::{json, Value};
use uuid::Uuid;

use crate::commands::adventure_api::rows::PurchaseParticipant;
use crate::support::instance_helpers::store;

pub(super) fn collect(
    client: &mut postgres::Client,
    player_uuid: Uuid,
    player_name: &str,
    include_party: bool,
) -> Result<Vec<PurchaseParticipant>, String> {
    if !include_party {
        return Ok(vec![buyer(player_uuid, player_name)]);
    }
    let party = store(lkjmc_store::party::current(client, player_uuid))?
        .ok_or_else(|| "buyer has no party".to_string())?;
    let members = store(lkjmc_store::party::members(client, party.id))?;
    if members.is_empty() {
        return Err("party has no members".to_string());
    }
    Ok(members
        .into_iter()
        .map(|member| PurchaseParticipant {
            player_uuid: member.player_uuid,
            player_name: member.player_name,
            role: if member.player_uuid == player_uuid {
                "buyer".to_string()
            } else {
                "member".to_string()
            },
        })
        .collect())
}

pub(super) fn as_json(participants: &[PurchaseParticipant]) -> Value {
    Value::Array(
        participants
            .iter()
            .map(|participant| {
                json!({
                    "playerUuid": participant.player_uuid.to_string(),
                    "playerName": &participant.player_name,
                    "role": &participant.role
                })
            })
            .collect(),
    )
}

pub(super) fn include_party(body: &Value) -> Result<bool, String> {
    match body.get("includeParty") {
        None => Ok(false),
        Some(value) => value
            .as_bool()
            .ok_or_else(|| "includeParty must be boolean".to_string()),
    }
}

fn buyer(player_uuid: Uuid, player_name: &str) -> PurchaseParticipant {
    PurchaseParticipant {
        player_uuid,
        player_name: player_name.to_string(),
        role: "buyer".to_string(),
    }
}
