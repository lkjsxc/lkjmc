use postgres::GenericClient;
use serde_json::Value;

use crate::error::StoreError;

pub(super) fn read(
    client: &mut impl GenericClient,
    domain: &str,
    key: &str,
) -> Result<Value, StoreError> {
    match domain {
        "permissions" => permissions(client, key),
        "claims" => claims(client, key),
        "profiles" => profiles(client, key),
        "presence" => presence(client, key),
        "routing" => routing(client),
        "settings" => settings(client, key),
        _ => Err(StoreError::invalid_state("unknown sync domain")),
    }
}

fn permissions(client: &mut impl GenericClient, key: &str) -> Result<Value, StoreError> {
    let (kind, id) = key
        .split_once(':')
        .ok_or_else(|| StoreError::invalid_state("invalid permissions key"))?;
    json(
        client,
        "select jsonb_build_object(
      'principalKind',$1::text,'principalId',$2::text,
      'grants',coalesce((select jsonb_agg(jsonb_build_object('id',g.id,'roleId',g.role_id,
        'expiresAt',g.expires_at) order by g.created_at,g.id) from admin_grants g
        where g.principal_kind=$1 and g.principal_id=$2 and g.revoked_at is null),'[]'::jsonb),
      'permissions',coalesce((select jsonb_agg(distinct permission order by permission)
        from admin_grants g join admin_roles r on r.id=g.role_id,
        jsonb_array_elements_text(r.permissions) permission
        where g.principal_kind=$1 and g.principal_id=$2 and g.revoked_at is null),
        '[]'::jsonb))",
        &[&kind, &id],
    )
}

fn claims(client: &mut impl GenericClient, key: &str) -> Result<Value, StoreError> {
    json(
        client,
        "select jsonb_build_object('chunks',coalesce(jsonb_agg(
      jsonb_build_object('claimId',c.id,'ownerUuid',c.owner_uuid,'ownerName',c.owner_name,
      'name',c.name,'worldName',ch.world_name,'chunkX',ch.chunk_x,'chunkZ',ch.chunk_z,
      'trusts',coalesce((select jsonb_agg(jsonb_build_object('uuid',t.trusted_uuid,
      'name',t.trusted_name) order by t.trusted_uuid) from claim_trusts t
      where t.claim_id=c.id),'[]'::jsonb)) order by c.name,ch.world_name,ch.chunk_x,ch.chunk_z)
      filter (where c.id is not null),'[]'::jsonb)) from claim_chunks ch
      join player_claims c on c.id=ch.claim_id and c.deleted_at is null where ch.instance_id=$1",
        &[&key],
    )
}

fn profiles(client: &mut impl GenericClient, key: &str) -> Result<Value, StoreError> {
    let (player, scope) = key
        .split_once(':')
        .ok_or_else(|| StoreError::invalid_state("invalid profile key"))?;
    let player = uuid::Uuid::parse_str(player)
        .map_err(|_| StoreError::invalid_state("invalid profile player UUID"))?;
    json(
        client,
        "select coalesce((select jsonb_build_object('playerUuid',player_uuid,
      'scope',scope,'profileRevision',revision,'schema','lkjmc-profile-one',
      'sha256',sha256,'envelope',envelope) from player_profile_snapshots
      where player_uuid=$1::uuid and scope=$2 order by revision desc limit 1),
      jsonb_build_object('playerUuid',$1::uuid,'scope',$2,'profile',null))",
        &[&player, &scope],
    )
}

fn presence(client: &mut impl GenericClient, key: &str) -> Result<Value, StoreError> {
    json(
        client,
        "select coalesce((select jsonb_build_object('instanceId',instance_id,
      'playerCount',player_count,'maxPlayers',max_players,'ready',ready,
      'lastHeartbeatAt',last_heartbeat_at,'suspendReason',suspend_reason)
      from instance_presence where instance_id=$1),
      jsonb_build_object('instanceId',$1::text,'available',false))",
        &[&key],
    )
}

fn routing(client: &mut impl GenericClient) -> Result<Value, StoreError> {
    json(
        client,
        "select jsonb_build_object('instances',coalesce(jsonb_agg(
      jsonb_build_object('id',i.id,'kind',i.kind,'desiredState',i.desired_state,
      'observedState',o.observed_state,'healthy',o.healthy,'ready',p.ready,
      'playerCount',p.player_count,'ports',coalesce((select jsonb_agg(jsonb_build_object(
      'port',ip.port,'purpose',ip.purpose) order by ip.port) from instance_ports ip
      where ip.instance_id=i.id),'[]'::jsonb)) order by i.id),'[]'::jsonb)) from instances i
      left join instance_observations o on o.instance_id=i.id
      left join instance_presence p on p.instance_id=i.id",
        &[],
    )
}

fn settings(client: &mut impl GenericClient, key: &str) -> Result<Value, StoreError> {
    let player = uuid::Uuid::parse_str(key)
        .map_err(|_| StoreError::invalid_state("invalid settings player UUID"))?;
    json(
        client,
        "select coalesce((select jsonb_build_object('playerUuid',player_uuid,
      'language',language,'menuEnabled',menu_enabled,'hudEnabled',hud_enabled,
      'tipsEnabled',tips_enabled,'privacy',privacy) from player_settings
      where player_uuid=$1::uuid),jsonb_build_object('playerUuid',$1::uuid,
      'language','en','menuEnabled',true,'hudEnabled',true,'tipsEnabled',true,
      'privacy','{}'::jsonb))",
        &[&player],
    )
}

fn json(
    client: &mut impl GenericClient,
    query: &str,
    params: &[&(dyn postgres::types::ToSql + Sync)],
) -> Result<Value, StoreError> {
    Ok(client.query_one(query, params)?.get(0))
}
