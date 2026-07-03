#[allow(dead_code)]
mod support;

use lkjmc_store::{claims, migrate, pool};
use std::env;
use uuid::Uuid;

#[test]
fn claims_round_trip_snapshot_and_delete() -> Result<(), lkjmc_store::error::StoreError> {
    let database_url = match env::var("LKJMC_STORE_TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut client = pool::connect(&database_url)?;
    let _schema = support::prepare_isolated_schema(&mut client)?;
    migrate::apply(&mut client)?;
    let claim_id = Uuid::new_v4();
    let owner = Uuid::new_v4();
    let trusted = Uuid::new_v4();
    claims::create_claim(&mut client, new_claim(claim_id, owner))?;
    claims::trust_player(&mut client, claim_id, trusted, "Friend")?;
    let owned = claims::list_claims_for_owner(&mut client, owner)?;
    assert_eq!(owned.len(), 1);
    assert_eq!(owned[0].name, "Base");
    let snapshot = claims::snapshot_claim_chunks(&mut client, "survival")?;
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].trusts[0].trusted_uuid, trusted);
    assert!(claims::lookup_claim_by_chunk(&mut client, "survival", "world", 1, 2)?.is_some());
    assert!(claims::delete_claim(&mut client, claim_id)?);
    assert!(claims::snapshot_claim_chunks(&mut client, "survival")?.is_empty());
    Ok(())
}

fn new_claim(claim_id: Uuid, owner: Uuid) -> claims::NewClaim<'static> {
    claims::NewClaim {
        id: claim_id,
        owner_uuid: owner,
        owner_name: "Owner",
        name: "Base",
        instance_id: "survival",
        world_name: "world",
        chunk_x: 1,
        chunk_z: 2,
    }
}
