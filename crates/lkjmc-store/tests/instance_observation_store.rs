#[allow(dead_code)]
mod support;

use lkjmc_store::{instance, migrate};
use serde_json::json;

#[test]
fn migrated_schema_accepts_kubernetes_observed_state() -> Result<(), lkjmc_store::error::StoreError>
{
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    instance::insert(client, "hub", None, "paper", "running", &json!({}))?;
    instance::upsert_observation(
        client,
        "hub",
        "kubernetes-ready",
        None,
        true,
        Some("ready pod observed"),
    )?;
    let stored = instance::get(client, "hub")?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("instance missing"))?;
    assert_eq!(stored.observed_state.as_deref(), Some("kubernetes-ready"));
    Ok(())
}
