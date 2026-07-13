use lkjmc_store::{migrate, player, player_session};
use serde_json::json;
use uuid::Uuid;

use super::support;

pub fn profile_json(points: i64) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "schema":"lkjmc-profile-one","inventory":[],"armor":[],"offhand":null,
        "selectedHotbarSlot":0,"enderChest":[],
        "experience":{"progress":0.0,"level":0,"total":0},
        "vitals":{"health":20.0,"food":20,"saturation":5.0,"air":300},
        "potionEffects":[],"gameMode":null,"pluginData":[],"homes":[],"warps":[],
        "points":points,"achievements":[],
        "settings":{"menuEnabled":true,"hudEnabled":true,"tipsEnabled":true,"privacy":"private"},
        "language":"en"
    }))
    .unwrap()
}

pub fn setup_profile(
    client: &mut postgres::Client,
) -> Result<(Uuid, Uuid, i64), lkjmc_store::error::StoreError> {
    migrate::apply(client)?;
    let player_id = Uuid::new_v4();
    player::insert_identity(client, player_id, "Player")?;
    let session = Uuid::new_v4();
    player_session::insert(client, session, player_id, "hub")?;
    let lease = player::acquire_lease(client, player_id, "profile", "hub", Uuid::new_v4())?;
    let body = profile_json(1);
    player::write_snapshot(
        client,
        player::NewSnapshot {
            id: Uuid::new_v4(),
            player_uuid: player_id,
            scope: "profile",
            session_id: session,
            expected_session_revision: 1,
            expected_lease_fence: lease.fence,
            expected_snapshot_revision: 0,
            correlation_id: Uuid::new_v4(),
            source_instance: "hub",
            profile_json: &body,
        },
    )?;
    Ok((player_id, session, lease.fence))
}

pub fn fail_feed(client: &mut postgres::Client, kind: &str) -> Result<(), postgres::Error> {
    client.batch_execute(&format!(
        "create or replace function fail_feed() returns trigger language plpgsql as $$
         begin if new.aggregate_kind = '{kind}' then raise exception 'deterministic failpoint'; end if;
         return new; end $$;
         create trigger workflow_failpoint before insert on workflow_change_feed
         for each row execute function fail_feed();"
    ))
}

pub fn database() -> Result<Option<support::TestDatabase>, lkjmc_store::error::StoreError> {
    support::database()
}
