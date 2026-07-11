use std::fs;

use lkjmc_core::id::InstanceId;
use lkjmc_core::temporary::{plan_temporary_instance, TemporaryInstanceRequest};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::commands::adventure_confirmation;
use crate::commands::temporary_api::create_support::{
    ensure_new_world, instance_config, runtime_facts,
};
use crate::commands::temporary_api::request;
use crate::dispatch as api;
use crate::support::audit_helpers::audit;
use crate::support::instance_helpers::{store, with_connection};

pub fn handle(
    state: &AppState,
    envelope: lkjmc_core::command::CommandEnvelope,
) -> lkjmc_core::command::CommandResponse {
    if !adventure_confirmation::accepted(&envelope.body) {
        return adventure_confirmation::required(envelope);
    }
    with_connection(state, envelope, |state, envelope, client| {
        let id = request::string(&envelope.body, "id")?;
        InstanceId::parse(id.clone()).map_err(|error| error.to_string())?;
        let cleanup_policy = request::cleanup_policy(&envelope.body)?;
        let world_root = request::optional_string(
            &envelope.body,
            "worldRoot",
            format!("{}/temporary-worlds", state.data_root()),
        );
        let max_lifetime = request::u32_field(&envelope.body, "maxLifetimeSeconds", 3600)?;
        let retention = request::u32_field(&envelope.body, "retentionSeconds", 600)?;
        let facts = runtime_facts(state, client)?;
        let plan = plan_temporary_instance(
            &TemporaryInstanceRequest {
                instance_id: &id,
                world_root: &world_root,
                max_lifetime_seconds: max_lifetime,
                retention_seconds: retention,
                cleanup_policy,
            },
            &facts,
        )
        .map_err(|error| format!("temporary plan failed: {error:?}"))?;
        ensure_new_world(&plan.world_path)?;
        let jar = store(lkjmc_store::jar::latest_matching(client, "folia"))?
            .ok_or_else(|| "Folia jar asset not found".to_string())?;
        let config = instance_config(state, &plan, &jar.id.to_string())?;
        fs::create_dir_all(&plan.world_path).map_err(|error| format!("create world: {error}"))?;
        let rows = create_rows(
            state,
            client,
            &envelope,
            &plan,
            &config,
            &jar.id.to_string(),
        );
        if let Err(error) = rows {
            let _ = fs::remove_dir_all(&plan.world_path);
            return Err(error);
        }
        audit(
            client,
            &envelope,
            "temporary.instance.create",
            "temporary-instance",
            &id,
            "succeeded",
        )?;
        Ok(api::ok(
            envelope,
            json!({"id": id, "serverPort": plan.server_port, "worldPath": plan.world_path}),
        ))
    })
}

fn create_rows(
    state: &AppState,
    client: &mut postgres::Client,
    envelope: &lkjmc_core::command::CommandEnvelope,
    plan: &lkjmc_core::temporary::TemporaryInstancePlan,
    config: &Value,
    jar_id: &str,
) -> Result<(), String> {
    let mut tx = client.transaction().map_err(|error| error.to_string())?;
    store(lkjmc_store::instance::insert(
        &mut tx,
        &plan.instance_id,
        None,
        "folia",
        "stopped",
        config,
    ))?;
    store(lkjmc_store::instance::reserve_port(
        &mut tx,
        &plan.instance_id,
        i32::from(plan.server_port),
        "server",
    ))?;
    store(lkjmc_store::instance::set_jar_asset(
        &mut tx,
        &plan.instance_id,
        uuid::Uuid::parse_str(jar_id).map_err(|error| error.to_string())?,
    ))?;
    store(lkjmc_store::temporary::insert_instance(
        &mut tx,
        lkjmc_store::temporary::NewTemporaryInstance {
            instance_id: &plan.instance_id,
            owner_kind: owner_kind(&envelope.body).as_str(),
            owner_id: owner_id(envelope).as_str(),
            visibility: &plan.visibility,
            world_path: &plan.world_path,
            server_port: i32::from(plan.server_port),
            max_lifetime_seconds: seconds(plan.max_lifetime_seconds)?,
            retention_seconds: seconds(plan.retention_seconds)?,
            cleanup_policy: plan.cleanup_policy.as_str(),
            lifecycle_state: "created",
            start_deadline_seconds: 120,
            metadata: metadata(&envelope.body),
        },
    ))?;
    tx.commit().map_err(|error| error.to_string())?;
    crate::templates::render_instance(state, &plan.instance_id, "folia", config).map(|_| ())
}

fn owner_kind(body: &Value) -> String {
    request::optional_string(body, "ownerKind", "operator".to_string())
}

fn owner_id(envelope: &lkjmc_core::command::CommandEnvelope) -> String {
    request::optional_string(&envelope.body, "ownerId", envelope.actor.name.clone())
}

fn metadata(body: &Value) -> Value {
    body.get("metadata").cloned().unwrap_or_else(|| json!({}))
}

fn seconds(value: u32) -> Result<i32, String> {
    i32::try_from(value).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use lkjmc_core::command::{Actor, ActorKind, CommandEnvelope};
    use lkjmc_core::id::CommandId;
    use serde_json::json;

    use super::*;

    #[test]
    fn create_rejects_before_database_work() -> Result<(), String> {
        for body in [json!({}), json!({"acceptMinecraftEula": false})] {
            let response = handle(&state(), request(body)?);
            assert!(!response.ok);
            assert!(response.body.is_none());
            assert_eq!(
                response.error.map(|error| (error.code, error.retryable)),
                Some((adventure_confirmation::CODE.to_string(), false))
            );
        }
        Ok(())
    }

    fn request(body: serde_json::Value) -> Result<CommandEnvelope, String> {
        Ok(CommandEnvelope {
            request_id: CommandId::parse("request id", "temporary.instance.create")
                .map_err(|error| error.to_string())?,
            actor: Actor {
                kind: ActorKind::Cli,
                name: "consent-test".to_string(),
            },
            command: "temporary.instance.create".to_string(),
            body,
        })
    }

    fn state() -> AppState {
        AppState::with_config_path(
            None,
            1,
            "/tmp/config".to_string(),
            "/tmp/logs".to_string(),
            "/tmp/jars".to_string(),
            "/tmp/data".to_string(),
            None,
            None,
            None,
        )
    }
}
