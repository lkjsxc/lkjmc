create table sync_domain_revisions (
    domain text not null check (domain in
      ('permissions','claims','menus','profiles','presence','routing','settings')),
    key text not null check (octet_length(key) between 1 and 256),
    revision bigint not null check (revision > 0),
    updated_at timestamptz not null default now(),
    primary key (domain, key)
);

create table sync_change_feed (
    feed_revision bigserial primary key,
    writer_xid xid8 not null default pg_current_xact_id(),
    domain text not null,
    key text not null,
    domain_revision bigint not null check (domain_revision > 0),
    created_at timestamptz not null default now(),
    unique (writer_xid, domain, key),
    unique (domain, key, domain_revision),
    foreign key (domain, key) references sync_domain_revisions (domain, key)
);
create index sync_change_feed_created_idx on sync_change_feed (created_at);

create table sync_retention_policy (
    singleton boolean primary key default true check (singleton),
    active_days integer not null check (active_days = 30)
);
insert into sync_retention_policy (active_days) values (30);

create function sync_touch(sync_domain text, sync_key text) returns void
language plpgsql as $$
declare next_revision bigint;
begin
    if sync_key is null or octet_length(sync_key) not between 1 and 256 then
        raise exception 'invalid sync key';
    end if;
    if exists (select 1 from sync_change_feed where writer_xid = pg_current_xact_id()
      and domain = sync_domain and key = sync_key) then return; end if;
    insert into sync_domain_revisions(domain, key, revision)
    values (sync_domain, sync_key, 1)
    on conflict (domain, key) do update
      set revision = sync_domain_revisions.revision + 1, updated_at = now()
    returning revision into next_revision;
    insert into sync_change_feed(domain, key, domain_revision)
    values (sync_domain, sync_key, next_revision);
end $$;

create function sync_row_key(row_data jsonb, key_kind text) returns text
language sql immutable as $$ select case
  when key_kind = 'global' then 'global'
  when key_kind = 'network' then 'network'
  when key_kind = 'player' then row_data->>'player_uuid'
  when key_kind = 'instance' then row_data->>'instance_id'
  when key_kind = 'profile' then (row_data->>'player_uuid') || ':' || (row_data->>'scope')
  when key_kind = 'principal' then
    (row_data->>'principal_kind') || ':' || (row_data->>'principal_id')
  else null end $$;

create function sync_touch_direct() returns trigger language plpgsql as $$
begin
    if TG_OP <> 'DELETE' then
      perform sync_touch(TG_ARGV[0], sync_row_key(to_jsonb(NEW), TG_ARGV[1]));
    end if;
    if TG_OP <> 'INSERT' then
      perform sync_touch(TG_ARGV[0], sync_row_key(to_jsonb(OLD), TG_ARGV[1]));
    end if;
    return coalesce(NEW, OLD);
end $$;

create function sync_touch_claim() returns trigger language plpgsql as $$
declare instance_key text;
begin
    for instance_key in select distinct instance_id from claim_chunks where claim_id in
      (coalesce(to_jsonb(NEW)->>'claim_id', to_jsonb(NEW)->>'id')::uuid,
       coalesce(to_jsonb(OLD)->>'claim_id', to_jsonb(OLD)->>'id')::uuid)
    loop perform sync_touch('claims', instance_key); end loop;
    return coalesce(NEW, OLD);
end $$;

create function sync_touch_role() returns trigger language plpgsql as $$
declare principal_key text;
begin
    for principal_key in select distinct principal_kind || ':' || principal_id
      from admin_grants where role_id in (NEW.id, OLD.id)
    loop perform sync_touch('permissions', principal_key); end loop;
    return coalesce(NEW, OLD);
end $$;

create trigger sync_admin_grants after insert or update or delete on admin_grants
for each row execute function sync_touch_direct('permissions', 'principal');
create trigger sync_admin_roles after update on admin_roles
for each row execute function sync_touch_role();
create trigger sync_player_claims after update on player_claims
for each row execute function sync_touch_claim();
create trigger sync_claim_chunks after insert or update or delete on claim_chunks
for each row execute function sync_touch_direct('claims', 'instance');
create trigger sync_claim_trusts after insert or update or delete on claim_trusts
for each row execute function sync_touch_claim();
create trigger sync_profiles after insert or update or delete on player_profile_snapshots
for each row execute function sync_touch_direct('profiles', 'profile');
create trigger sync_presence after insert or update or delete on instance_presence
for each row execute function sync_touch_direct('presence', 'instance');
create trigger sync_settings after insert or update or delete on player_settings
for each row execute function sync_touch_direct('settings', 'player');

create trigger sync_routing_instances after insert or update or delete on instances
for each row execute function sync_touch_direct('routing', 'network');
create trigger sync_routing_observations after insert or update or delete on instance_observations
for each row execute function sync_touch_direct('routing', 'network');
create trigger sync_routing_ports after insert or update or delete on instance_ports
for each row execute function sync_touch_direct('routing', 'network');

create trigger sync_menus_shop after insert or update or delete on shop_items
for each row execute function sync_touch_direct('menus', 'global');
create trigger sync_menus_kits after insert or update or delete on kit_definitions
for each row execute function sync_touch_direct('menus', 'global');
create trigger sync_menus_votes after insert or update or delete on vote_links
for each row execute function sync_touch_direct('menus', 'global');
create trigger sync_menus_plugins after insert or update or delete on plugin_catalog_entries
for each row execute function sync_touch_direct('menus', 'global');

select sync_touch('menus', 'global');
select sync_touch('routing', 'network');
insert into sync_domain_revisions(domain, key, revision)
select 'permissions', principal_kind || ':' || principal_id, 1 from admin_grants
group by principal_kind, principal_id on conflict do nothing;
insert into sync_domain_revisions(domain, key, revision)
select 'claims', instance_id, 1 from claim_chunks group by instance_id on conflict do nothing;
insert into sync_domain_revisions(domain, key, revision)
select 'profiles', player_uuid::text || ':' || scope, max(revision)
from player_profile_snapshots group by player_uuid, scope on conflict do nothing;
insert into sync_domain_revisions(domain, key, revision)
select 'presence', instance_id, 1 from instance_presence on conflict do nothing;
insert into sync_domain_revisions(domain, key, revision)
select 'settings', player_uuid::text, 1 from player_settings on conflict do nothing;
