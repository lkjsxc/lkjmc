#[allow(dead_code)]
mod support;

use lkjmc_store::{migrate, sync};

#[test]
fn routing_revision_matches_every_payload_dependency() -> Result<(), lkjmc_store::error::StoreError>
{
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    let mut routing = revision(client, "routing", "network")?;

    client.execute(
        "insert into instances(id,kind,desired_state,config) values('hub','paper','running','{}')",
        &[],
    )?;
    routing = assert_routing(client, routing, |payload| {
        payload["instances"][0]["id"] == "hub"
    })?;

    client.execute(
        "insert into instance_observations(instance_id,observed_state,healthy)
         values('hub','process-healthy',true)",
        &[],
    )?;
    routing = assert_routing(client, routing, |payload| {
        payload["instances"][0]["observedState"] == "process-healthy"
    })?;

    client.execute(
        "insert into instance_ports(port,instance_id,purpose) values(25565,'hub','minecraft')",
        &[],
    )?;
    routing = assert_routing(client, routing, |payload| {
        payload["instances"][0]["ports"][0]["port"] == 25565
    })?;

    client.execute(
        "insert into instance_presence(instance_id,last_heartbeat_at,player_count,max_players,
         ready,metadata) values('hub',now(),1,20,true,'{}')",
        &[],
    )?;
    routing = assert_routing(client, routing, |payload| {
        payload["instances"][0]["playerCount"] == 1
    })?;
    let presence = revision(client, "presence", "hub")?;
    client.execute(
        "update instance_presence set player_count=9 where instance_id='hub'",
        &[],
    )?;
    assert_eq!(revision(client, "presence", "hub")?, presence + 1);
    routing = assert_routing(client, routing, |payload| {
        payload["instances"][0]["playerCount"] == 9
    })?;

    let mut tx = client.transaction()?;
    tx.execute(
        "update instance_presence set player_count=3 where instance_id='hub'",
        &[],
    )?;
    tx.rollback()?;
    assert_eq!(revision(client, "routing", "network")?, routing);
    assert_eq!(routing_player_count(client)?, 9);
    Ok(())
}

#[test]
fn retention_archives_then_deletes_in_bounded_runs() -> Result<(), lkjmc_store::error::StoreError> {
    let Some(mut database) = support::database()? else {
        return Ok(());
    };
    let client = database.client_mut();
    migrate::apply(client)?;
    client.execute(
        "insert into instances(id,kind,desired_state,config)
         values('retention-probe','paper','stopped','{}')",
        &[],
    )?;
    client.execute(
        "update sync_change_feed set created_at=now()-interval '31 days'",
        &[],
    )?;
    client.execute(
        "insert into sync_change_archive(feed_revision,writer_xid,domain,key,
         domain_revision,created_at) select feed_revision,writer_xid,domain,key,
         domain_revision,created_at from sync_change_feed order by feed_revision limit 1",
        &[],
    )?;
    let result = sync::run_retention(client)?;
    assert!(result.archived > 0);
    assert_eq!(result.deleted, 0);
    let active: i64 = client
        .query_one("select count(*) from sync_change_feed", &[])?
        .get(0);
    let archived: i64 = client
        .query_one("select count(*) from sync_change_archive", &[])?
        .get(0);
    assert_eq!(active, 0);
    assert_eq!(archived as u64, result.archived + 1);
    client.execute(
        "update sync_change_archive set created_at=now()-interval '366 days'",
        &[],
    )?;
    let result = sync::run_retention(client)?;
    assert_eq!(result.deleted, archived as u64);
    Ok(())
}

fn assert_routing(
    client: &mut postgres::Client,
    before: i64,
    predicate: impl FnOnce(&serde_json::Value) -> bool,
) -> Result<i64, lkjmc_store::error::StoreError> {
    let sync::SnapshotResult::Available(snapshot) = sync::snapshot(client, "routing", "network")?
    else {
        return Err(lkjmc_store::error::StoreError::invalid_state(
            "routing unavailable",
        ));
    };
    assert_eq!(snapshot.revision, before + 1);
    assert!(
        predicate(&snapshot.payload),
        "routing payload input changed without coherence"
    );
    Ok(snapshot.revision)
}

fn routing_player_count(
    client: &mut postgres::Client,
) -> Result<i64, lkjmc_store::error::StoreError> {
    let sync::SnapshotResult::Available(snapshot) = sync::snapshot(client, "routing", "network")?
    else {
        return Err(lkjmc_store::error::StoreError::invalid_state(
            "routing unavailable",
        ));
    };
    Ok(snapshot.payload["instances"][0]["playerCount"]
        .as_i64()
        .unwrap_or(-1))
}

fn revision(
    client: &mut postgres::Client,
    domain: &str,
    key: &str,
) -> Result<i64, lkjmc_store::error::StoreError> {
    Ok(client
        .query_one(
            "select revision from sync_domain_revisions where domain=$1 and key=$2",
            &[&domain, &key],
        )?
        .get(0))
}
