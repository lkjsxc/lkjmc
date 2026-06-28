use std::fs;
use std::path::Path;

use lkjmc_core::command::{CommandEnvelope, CommandResponse};
use lkjmc_core::id::InstanceId;
use lkjmc_core::temporary::{plan_temporary_instance, CleanupPolicy, TemporaryInstanceRequest};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::adventure_api::rows::{insert_purchase, PurchaseRows};
use crate::api;
use crate::app::AppState;
use crate::audit_helpers::audit;
use crate::instance_helpers::{body_string, store, with_client};
use crate::temporary_api::create_support::{
    ensure_new_world, instance_config, read_forwarding_secret, runtime_facts,
};
use crate::temporary_api::lifecycle::start_ready;
use crate::temporary_api::request;

pub fn end(state: &AppState, envelope: CommandEnvelope) -> CommandResponse {
    with_client(state, envelope, |state, envelope, client| {
        request::require_eula(&envelope.body)?;
        let player_uuid = parse_uuid(&envelope.body, "playerUuid")?;
        let player_name = body_string(&envelope.body, "playerName")?;
        let cost = i64::from(request::u32_field(&envelope.body, "cost", 100)?);
        if cost <= 0 {
            return Err("cost must be positive".to_string());
        }
        let session_id = Uuid::new_v4();
        let instance_id = instance_id(session_id)?;
        let world_root = request::optional_string(
            &envelope.body,
            "worldRoot",
            format!("{}/temporary-worlds", state.data_root()),
        );
        let max_lifetime_seconds = request::u32_field(&envelope.body, "maxLifetimeSeconds", 3600)?;
        let retention_seconds = request::u32_field(&envelope.body, "retentionSeconds", 600)?;
        seconds(max_lifetime_seconds)?;
        seconds(retention_seconds)?;
        let plan = plan_temporary_instance(
            &TemporaryInstanceRequest {
                instance_id: &instance_id,
                world_root: &world_root,
                max_lifetime_seconds,
                retention_seconds,
                cleanup_policy: CleanupPolicy::Delete,
            },
            &runtime_facts(state, client)?,
        )
        .map_err(|error| format!("temporary plan failed: {error:?}"))?;
        ensure_new_world(&plan.world_path)?;
        let jar = store(lkjmc_store::jar::latest_matching(client, "folia"))?
            .ok_or_else(|| "Folia jar asset not found".to_string())?;
        let secret = read_forwarding_secret(state)?;
        let config = instance_config(state, &plan, &jar.id.to_string(), &secret)?;
        prepare_files(state, &plan, &config)?;
        let session_id_text = session_id.to_string();
        let rows = PurchaseRows {
            session_id,
            session_id_text: &session_id_text,
            player_uuid,
            player_name: &player_name,
            cost,
            plan: &plan,
            config: &config,
            jar_id: jar.id,
        };
        let ledger = match insert_purchase(client, rows) {
            Ok(ledger) => ledger,
            Err(error) => {
                cleanup_files(state, &plan.instance_id, &plan.world_path);
                return Err(error);
            }
        };
        if let Err(error) = start_ready(state, client, &plan.instance_id, 180) {
            refund_purchase(client, session_id, player_uuid, cost, &error)?;
            audit(
                client,
                &envelope,
                "adventure.end.purchase",
                "adventure-session",
                &session_id.to_string(),
                "failed",
            )?;
            return Err(format!("{error}; points refunded"));
        }
        store(lkjmc_store::temporary::update_session_state(
            client, session_id, "ready", None, None,
        ))?;
        audit(
            client,
            &envelope,
            "adventure.end.purchase",
            "adventure-session",
            &session_id.to_string(),
            "succeeded",
        )?;
        Ok(api::ok(
            envelope,
            json!({
                "sessionId": session_id.to_string(),
                "temporaryInstanceId": plan.instance_id,
                "pointsLedgerId": ledger.to_string(),
                "targetServer": plan.instance_id,
                "state": "ready"
            }),
        ))
    })
}

fn refund_purchase(
    client: &mut postgres::Client,
    session_id: Uuid,
    player_uuid: Uuid,
    cost: i64,
    reason: &str,
) -> Result<(), String> {
    let refund = store(lkjmc_store::points::grant_with_correlation(
        client,
        player_uuid,
        cost,
        "end-expedition-refund",
        Some(session_id),
    ))?;
    store(lkjmc_store::temporary::update_session_state(
        client,
        session_id,
        "refunded",
        Some(reason),
        Some(refund),
    ))
}

fn prepare_files(
    state: &AppState,
    plan: &lkjmc_core::temporary::TemporaryInstancePlan,
    config: &Value,
) -> Result<(), String> {
    fs::create_dir_all(&plan.world_path).map_err(|error| format!("create world: {error}"))?;
    crate::templates::render_instance(state, &plan.instance_id, "folia", config).map(|_| ())
}

fn cleanup_files(state: &AppState, id: &str, world_path: &str) {
    let _ = fs::remove_dir_all(world_path);
    let _ = fs::remove_dir_all(Path::new(&state.data_root()).join(id));
}

fn parse_uuid(body: &Value, field: &'static str) -> Result<Uuid, String> {
    Uuid::parse_str(&body_string(body, field)?).map_err(|error| error.to_string())
}

fn seconds(value: u32) -> Result<i32, String> {
    i32::try_from(value).map_err(|error| error.to_string())
}

fn instance_id(session_id: Uuid) -> Result<String, String> {
    let suffix = session_id.simple().to_string();
    let id = format!("end-{}", &suffix[..12]);
    InstanceId::parse(id.clone()).map_err(|error| error.to_string())?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_instance_id_is_valid() {
        let id = instance_id(Uuid::nil());
        assert!(id.is_ok(), "{id:?}");
        let Ok(value) = id else {
            return;
        };
        assert!(value.starts_with("end-"));
    }
}
