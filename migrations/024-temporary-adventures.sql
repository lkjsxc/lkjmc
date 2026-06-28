create table if not exists temporary_instances (
    instance_id text primary key references instances(id) on delete cascade,
    owner_kind text not null,
    owner_id text not null,
    visibility text not null,
    world_path text not null unique,
    server_port integer not null unique,
    max_lifetime_seconds integer not null check (max_lifetime_seconds > 0),
    retention_seconds integer not null check (retention_seconds >= 0),
    cleanup_policy text not null,
    lifecycle_state text not null,
    start_deadline_at timestamptz not null,
    stop_deadline_at timestamptz not null,
    expires_at timestamptz not null,
    retain_until timestamptz not null,
    cleanup_attempts integer not null default 0,
    last_error text,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint temporary_instances_visibility_check
        check (visibility in ('hidden', 'listed')),
    constraint temporary_instances_cleanup_policy_check
        check (cleanup_policy in ('delete', 'archive')),
    constraint temporary_instances_lifecycle_state_check check (
        lifecycle_state in ('planned', 'created', 'starting', 'ready',
        'stopping', 'stopped', 'failed', 'cleaning', 'cleaned', 'archived')
    )
);

create index if not exists temporary_instances_state_idx
    on temporary_instances (lifecycle_state, expires_at);

create table if not exists adventure_sessions (
    id uuid primary key,
    adventure_kind text not null,
    buyer_uuid uuid not null,
    buyer_name text not null,
    temporary_instance_id text not null
        references temporary_instances(instance_id) on delete restrict,
    points_cost bigint not null check (points_cost >= 0),
    points_ledger_id uuid references points_ledger(id),
    refund_ledger_id uuid references points_ledger(id),
    state text not null,
    start_deadline_at timestamptz not null,
    stop_deadline_at timestamptz not null,
    failure_reason text,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint adventure_sessions_state_check check (
        state in ('pending', 'starting', 'ready', 'active', 'completed',
        'failed', 'refunded', 'cancelled', 'expired')
    )
);

create index if not exists adventure_sessions_buyer_idx
    on adventure_sessions (buyer_uuid, created_at desc);

create table if not exists adventure_participants (
    session_id uuid not null references adventure_sessions(id) on delete cascade,
    player_uuid uuid not null,
    player_name text not null,
    role text not null,
    state text not null,
    joined_at timestamptz,
    left_at timestamptz,
    metadata jsonb not null default '{}'::jsonb,
    primary key (session_id, player_uuid),
    constraint adventure_participants_state_check
        check (state in ('invited', 'queued', 'joined', 'left', 'failed'))
);

create table if not exists adventure_cleanup_events (
    id uuid primary key,
    temporary_instance_id text not null
        references temporary_instances(instance_id) on delete cascade,
    event_kind text not null,
    result text not null,
    diagnostic text,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);

create index if not exists adventure_cleanup_events_instance_idx
    on adventure_cleanup_events (temporary_instance_id, created_at desc);
