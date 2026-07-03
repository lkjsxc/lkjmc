use std::collections::BTreeMap;

use lkjmc_core::claim::ClaimName;
use postgres::{Client, GenericClient};
use uuid::Uuid;

use crate::error::StoreError;

pub use crate::claims_types::{ClaimChunkRecord, ClaimSummary, NewClaim, TrustedPlayer};

pub fn create_claim(client: &mut Client, claim: NewClaim<'_>) -> Result<Uuid, StoreError> {
    let mut transaction = client.transaction()?;
    let claim_id = create_claim_in(&mut transaction, claim)?;
    transaction.commit()?;
    Ok(claim_id)
}

pub fn create_claim_in(
    client: &mut impl GenericClient,
    claim: NewClaim<'_>,
) -> Result<Uuid, StoreError> {
    let name = ClaimName::parse(claim.name)
        .map_err(|error| StoreError::invalid_state(error.to_string()))?;
    client.execute(
        "insert into player_claims (id, owner_uuid, owner_name, name, name_key) values ($1, $2, $3, $4, $5)",
        &[&claim.id, &claim.owner_uuid, &claim.owner_name, &name.value(), &name.key()],
    )?;
    client.execute(
        "insert into claim_chunks (claim_id, instance_id, world_name, chunk_x, chunk_z) values ($1, $2, $3, $4, $5)",
        &[&claim.id, &claim.instance_id, &claim.world_name, &claim.chunk_x, &claim.chunk_z],
    )?;
    Ok(claim.id)
}

pub fn delete_claim(client: &mut Client, claim_id: Uuid) -> Result<bool, StoreError> {
    let mut transaction = client.transaction()?;
    transaction.execute("delete from claim_chunks where claim_id = $1", &[&claim_id])?;
    transaction.execute("delete from claim_trusts where claim_id = $1", &[&claim_id])?;
    let changed = transaction.execute(
        "update player_claims set deleted_at = now() where id = $1 and deleted_at is null",
        &[&claim_id],
    )?;
    transaction.commit()?;
    Ok(changed > 0)
}

pub fn trust_player(
    client: &mut Client,
    claim_id: Uuid,
    trusted_uuid: Uuid,
    trusted_name: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into claim_trusts (claim_id, trusted_uuid, trusted_name) values ($1, $2, $3)
         on conflict (claim_id, trusted_uuid) do update set trusted_name = excluded.trusted_name",
        &[&claim_id, &trusted_uuid, &trusted_name],
    )?;
    Ok(())
}

pub fn untrust_player(
    client: &mut Client,
    claim_id: Uuid,
    trusted_uuid: Uuid,
) -> Result<u64, StoreError> {
    Ok(client.execute(
        "delete from claim_trusts where claim_id = $1 and trusted_uuid = $2",
        &[&claim_id, &trusted_uuid],
    )?)
}

pub fn list_claims_for_owner(
    client: &mut Client,
    owner_uuid: Uuid,
) -> Result<Vec<ClaimSummary>, StoreError> {
    let rows = client.query(
        "select c.id, c.owner_uuid, c.owner_name, c.name, count(ch.claim_id)::bigint
         from player_claims c left join claim_chunks ch on ch.claim_id = c.id
         where c.owner_uuid = $1 and c.deleted_at is null
         group by c.id order by c.name",
        &[&owner_uuid],
    )?;
    Ok(rows.into_iter().map(summary_from_row).collect())
}

pub use snapshot_claim_chunks as list_claims_for_instance;

pub fn snapshot_claim_chunks(
    client: &mut Client,
    instance_id: &str,
) -> Result<Vec<ClaimChunkRecord>, StoreError> {
    let rows = client.query(
        "select c.id, c.owner_uuid, c.owner_name, c.name, ch.instance_id, ch.world_name, ch.chunk_x, ch.chunk_z,
         t.trusted_uuid, t.trusted_name
         from claim_chunks ch join player_claims c on c.id = ch.claim_id
         left join claim_trusts t on t.claim_id = c.id
         where ch.instance_id = $1 and c.deleted_at is null
         order by c.name, ch.world_name, ch.chunk_x, ch.chunk_z",
        &[&instance_id],
    )?;
    Ok(group_chunks(rows))
}

pub fn lookup_claim_by_chunk(
    client: &mut Client,
    instance_id: &str,
    world_name: &str,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<Option<ClaimChunkRecord>, StoreError> {
    Ok(snapshot_claim_chunks(client, instance_id)?
        .into_iter()
        .find(|claim| {
            claim.world_name == world_name && claim.chunk_x == chunk_x && claim.chunk_z == chunk_z
        }))
}

pub fn active_claim_by_owner_name(
    client: &mut Client,
    owner_uuid: Uuid,
    name: &str,
) -> Result<Option<ClaimSummary>, StoreError> {
    let name =
        ClaimName::parse(name).map_err(|error| StoreError::invalid_state(error.to_string()))?;
    let row = client.query_opt(
        "select c.id, c.owner_uuid, c.owner_name, c.name, count(ch.claim_id)::bigint
         from player_claims c left join claim_chunks ch on ch.claim_id = c.id
         where c.owner_uuid = $1 and c.name_key = $2 and c.deleted_at is null group by c.id",
        &[&owner_uuid, &name.key()],
    )?;
    Ok(row.map(summary_from_row))
}

fn summary_from_row(row: postgres::Row) -> ClaimSummary {
    ClaimSummary {
        id: row.get(0),
        owner_uuid: row.get(1),
        owner_name: row.get(2),
        name: row.get(3),
        chunk_count: row.get(4),
    }
}

fn group_chunks(rows: Vec<postgres::Row>) -> Vec<ClaimChunkRecord> {
    let mut claims = BTreeMap::<(Uuid, String, i32, i32), ClaimChunkRecord>::new();
    for row in rows {
        let key = (row.get(0), row.get(5), row.get(6), row.get(7));
        let entry = claims.entry(key).or_insert_with(|| chunk_from_row(&row));
        let trusted_uuid: Option<Uuid> = row.get(8);
        if let Some(trusted_uuid) = trusted_uuid {
            entry.trusts.push(TrustedPlayer {
                trusted_uuid,
                trusted_name: row.get(9),
            });
        }
    }
    claims.into_values().collect()
}

fn chunk_from_row(row: &postgres::Row) -> ClaimChunkRecord {
    ClaimChunkRecord {
        claim_id: row.get(0),
        owner_uuid: row.get(1),
        owner_name: row.get(2),
        name: row.get(3),
        instance_id: row.get(4),
        world_name: row.get(5),
        chunk_x: row.get(6),
        chunk_z: row.get(7),
        trusts: Vec::new(),
    }
}
