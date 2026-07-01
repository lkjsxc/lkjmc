create table if not exists random_teleports (
    id uuid primary key,
    correlation_id uuid not null unique,
    player_uuid uuid not null references player_identities(player_uuid),
    server_id text not null,
    world text not null,
    x double precision not null,
    y double precision not null,
    z double precision not null,
    cost_points bigint not null,
    state text not null,
    failure_reason text,
    created_at timestamptz not null default now(),
    completed_at timestamptz,
    refunded_at timestamptz,
    metadata jsonb not null default '{}'::jsonb
);

create index if not exists random_teleports_player_idx
    on random_teleports (player_uuid, created_at desc);

create index if not exists random_teleports_live_idx
    on random_teleports (player_uuid, server_id, created_at desc)
    where state in ('reserved', 'completed');
