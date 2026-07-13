create table player_profile_snapshot_quarantine (
    quarantine_id bigserial primary key,
    legacy_snapshot_id uuid not null,
    player_uuid uuid not null,
    scope text not null,
    legacy_revision bigint not null,
    legacy_format text not null,
    legacy_payload bytea not null,
    legacy_sha256 text not null,
    reason text not null check (reason = 'untyped-profile'),
    quarantined_at timestamptz not null default now()
);

insert into player_profile_snapshot_quarantine
    (legacy_snapshot_id, player_uuid, scope, legacy_revision, legacy_format,
     legacy_payload, legacy_sha256, reason)
select id, player_uuid, scope, revision, payload_format, payload, sha256,
       'untyped-profile'
from player_profile_snapshots;
revoke all on player_profile_snapshot_quarantine from public;

drop table player_profile_snapshots;
create table player_profile_snapshots (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    scope text not null,
    revision bigint not null check (revision > 0),
    session_id uuid not null references player_sessions(id),
    session_revision bigint not null check (session_revision > 0),
    lease_fence bigint not null check (lease_fence > 0),
    expected_snapshot_revision bigint not null check (expected_snapshot_revision >= 0),
    correlation_id uuid not null unique,
    schema_name text not null check (schema_name = 'lkjmc-profile-one'),
    envelope jsonb not null check (jsonb_typeof(envelope) = 'object'),
    canonical_json bytea not null check (octet_length(canonical_json) <= 1048576),
    sha256 text not null check (sha256 ~ '^[0-9a-f]{64}$'),
    source_instance text not null,
    created_at timestamptz not null default now(),
    unique (player_uuid, scope, revision)
);
create index player_profile_snapshots_player_idx
    on player_profile_snapshots (player_uuid, scope, revision desc);

alter table player_identities add column revision bigint not null default 1
    check (revision > 0);
alter table player_sessions add column revision bigint not null default 1
    check (revision > 0);
alter table player_sessions add column correlation_id uuid unique;

drop table player_profile_leases;
create table player_profile_leases (
    player_uuid uuid not null references player_identities(player_uuid),
    scope text not null,
    holder text not null,
    fence bigint not null check (fence > 0),
    expires_at timestamptz not null,
    correlation_id uuid not null unique,
    updated_at timestamptz not null default now(),
    primary key (player_uuid, scope)
);

drop table temporary_transfer_intents;

create table transfer_workflows (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    session_id uuid not null references player_sessions(id),
    session_revision bigint not null check (session_revision > 0),
    profile_revision bigint not null check (profile_revision > 0),
    lease_fence bigint not null check (lease_fence > 0),
    scope text not null,
    target_server text not null,
    state text not null check (state in
      ('pending_save','save_acknowledged','pending_arrival','arrived','failed')),
    revision bigint not null check (revision > 0),
    fence bigint not null check (fence > 0),
    correlation_id uuid not null unique,
    failure_reason text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table item_delivery_workflows (
    id uuid primary key,
    purchase_id uuid not null unique references shop_purchases(id),
    player_uuid uuid not null references player_identities(player_uuid),
    delivery jsonb not null,
    state text not null check (state in ('pending_receipt','received','failed')),
    revision bigint not null check (revision > 0),
    fence bigint not null check (fence > 0),
    correlation_id uuid not null unique,
    failure_reason text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table runtime_effect_workflows (
    id uuid primary key,
    instance_id text not null references instances(id),
    effect_kind text not null,
    requested_state jsonb not null,
    state text not null check (state in ('pending_observation','observed','failed')),
    revision bigint not null check (revision > 0),
    fence bigint not null check (fence > 0),
    correlation_id uuid not null unique,
    failure_reason text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

alter table adventure_sessions drop constraint adventure_sessions_state_check;
update adventure_sessions
set state = case
    when state in ('pending', 'starting') then 'pending_start'
    when state in ('ready', 'active') then 'start_observed'
    else 'failed'
end,
failure_reason = case
    when state in ('pending', 'starting', 'ready', 'active') then failure_reason
    else coalesce(failure_reason, '045-cutover')
end;
alter table adventure_sessions add constraint adventure_sessions_state_check
    check (state in ('pending_start','start_observed','pending_cleanup','cleaned','failed'));
alter table adventure_sessions add column revision bigint not null default 1 check (revision > 0);
alter table adventure_sessions add column fence bigint not null default 1 check (fence > 0);
alter table adventure_sessions add column correlation_id uuid;
update adventure_sessions set correlation_id = id where correlation_id is null;
alter table adventure_sessions alter column correlation_id set not null;
alter table adventure_sessions add unique (correlation_id);

create table workflow_change_feed (
    feed_revision bigserial primary key,
    aggregate_kind text not null check (aggregate_kind in
      ('profile','transfer','delivery','adventure','runtime')),
    aggregate_id uuid not null,
    aggregate_revision bigint not null check (aggregate_revision > 0),
    correlation_id uuid not null,
    state text not null,
    fact jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    unique (aggregate_kind, aggregate_id, aggregate_revision)
);
create index workflow_change_feed_created_idx on workflow_change_feed (created_at);
insert into workflow_change_feed
    (aggregate_kind, aggregate_id, aggregate_revision, correlation_id, state)
select 'adventure', id, revision, correlation_id, state
from adventure_sessions;

create table workflow_change_archive (
    feed_revision bigint primary key,
    aggregate_kind text not null,
    aggregate_id uuid not null,
    aggregate_revision bigint not null,
    correlation_id uuid not null,
    state text not null,
    fact jsonb not null,
    created_at timestamptz not null,
    archived_at timestamptz not null default now()
);

create table workflow_retention_policy (
    singleton boolean primary key default true check (singleton),
    active_days integer not null check (active_days = 30),
    archive_days integer not null check (archive_days = 365)
);
insert into workflow_retention_policy (active_days, archive_days) values (30, 365);
