#[allow(dead_code)]
mod support;

use lkjmc_store::{claims, migrate};
use uuid::Uuid;

#[test]
fn claims_round_trip_snapshot_and_delete() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let claim_id = Uuid::new_v4();
    let owner = Uuid::new_v4();
    let trusted = Uuid::new_v4();
    claims::create_claim(client, new_claim(claim_id, owner))?;
    claims::trust_player(client, claim_id, trusted, "Friend")?;
    let owned = claims::list_claims_for_owner(client, owner)?;
    assert_eq!(owned.len(), 1);
    assert_eq!(owned[0].name, "Base");
    let snapshot = claims::snapshot_claim_chunks(client, "survival")?;
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].trusts[0].trusted_uuid, trusted);
    assert!(claims::lookup_claim_by_chunk(client, "survival", "world", 1, 2)?.is_some());
    assert!(claims::delete_claim(client, claim_id)?);
    assert!(claims::snapshot_claim_chunks(client, "survival")?.is_empty());
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
