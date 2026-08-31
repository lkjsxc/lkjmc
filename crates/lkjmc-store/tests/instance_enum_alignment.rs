#[allow(dead_code)]
mod support;

use lkjmc_core::instance::{DesiredState, InstanceKind};
use lkjmc_store::{instance, migrate};
use serde_json::json;

#[test]
fn postgres_constraints_accept_exact_core_instance_enums(
) -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;

    for (index, (kind, wire)) in InstanceKind::ALL.iter().enumerate() {
        assert_eq!(kind.as_str(), *wire);
        instance::insert(
            client,
            &format!("kind-{index}"),
            None,
            wire,
            DesiredState::Stopped.as_str(),
            &json!({}),
        )?;
    }
    for (index, (state, wire)) in DesiredState::ALL.iter().enumerate() {
        assert_eq!(state.as_str(), *wire);
        instance::insert(
            client,
            &format!("state-{index}"),
            None,
            InstanceKind::Paper.as_str(),
            wire,
            &json!({}),
        )?;
    }

    assert!(instance::insert(
        client,
        "invalid-kind",
        None,
        "hub",
        DesiredState::Stopped.as_str(),
        &json!({}),
    )
    .is_err());
    assert!(instance::insert(
        client,
        "invalid-state",
        None,
        InstanceKind::Paper.as_str(),
        "ready",
        &json!({}),
    )
    .is_err());

    let count: i64 = client
        .query_one("select count(*) from instances", &[])?
        .get(0);
    assert_eq!(
        count,
        (InstanceKind::ALL.len() + DesiredState::ALL.len()) as i64
    );
    Ok(())
}
