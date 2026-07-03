use serde_json::json;
use uuid::Uuid;

use crate::api;
use crate::app::AppState;
use crate::audit_helpers::audit;
use crate::instance_helpers::{body_string, store, with_connection};

pub fn intent(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    with_connection(state, envelope, |_state, envelope, client| {
        let player_uuid = parse_uuid(&envelope, "playerUuid")?;
        let player_name = body_string(&envelope.body, "playerName")?;
        let instance_id = body_string(&envelope.body, "temporaryInstanceId")?;
        let temp = store(lkjmc_store::temporary::get_instance(client, &instance_id))?
            .ok_or_else(|| format!("temporary instance not found: {instance_id}"))?;
        if temp.lifecycle_state != "ready" {
            return Err(format!(
                "temporary instance is not ready: {}",
                temp.lifecycle_state
            ));
        }
        store(lkjmc_store::player::insert_identity(
            client,
            player_uuid,
            &player_name,
        ))?;
        let intent_id = Uuid::new_v4();
        store(lkjmc_store::temporary::create_intent(
            client,
            lkjmc_store::temporary::NewTransferIntent {
                id: intent_id,
                temporary_instance_id: &instance_id,
                player_uuid,
                player_name: &player_name,
                expires_in_seconds: 30,
                metadata: json!({}),
            },
        ))?;
        audit(
            client,
            &envelope,
            "temporary.transfer.intent",
            "temporary-instance",
            &instance_id,
            "succeeded",
        )?;
        Ok(api::ok(
            envelope,
            json!({
                "intentId": intent_id.to_string(),
                "targetServer": instance_id,
                "expiresInSeconds": 30
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
