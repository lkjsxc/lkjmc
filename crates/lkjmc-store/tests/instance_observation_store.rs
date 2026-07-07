#[allow(dead_code)]
mod support;

use lkjmc_store::{instance, migrate, pool};
use serde_json::json;
use std::env;

#[test]
fn migrated_schema_accepts_kubernetes_observed_state() -> Result<(), lkjmc_store::error::StoreError>
{
    let database_url = match env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut client = pool::connect(&database_url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    instance::insert(&mut client, "hub", None, "paper", "running", &json!({}))?;
    instance::upsert_observation(
        &mut client,
        "hub",
        "kubernetes-ready",
        None,
        true,
        Some("ready pod observed"),
    )?;
    let stored = instance::get(&mut client, "hub")?
        .ok_or_else(|| lkjmc_store::error::StoreError::invalid_state("instance missing"))?;
    assert_eq!(stored.observed_state.as_deref(), Some("kubernetes-ready"));
    Ok(())
}
