create table if not exists player_identities (
    player_uuid uuid primary key,
    current_name text not null,
    first_seen_at timestamptz not null default now(),
    last_seen_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb
);

create table if not exists player_sessions (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    current_server text,
    joined_at timestamptz not null default now(),
    left_at timestamptz,
    transfer_correlation_id uuid,
    metadata jsonb not null default '{}'::jsonb
);

create table if not exists player_profile_leases (
    player_uuid uuid not null references player_identities(player_uuid),
    scope text not null,
    holder text not null,
    revision bigint not null,
    expires_at timestamptz not null,
    updated_at timestamptz not null default now(),
    primary key (player_uuid, scope),
    constraint player_profile_leases_revision_check check (revision >= 0)
);

create table if not exists player_profile_snapshots (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    scope text not null,
    revision bigint not null,
    payload_format text not null,
    payload bytea not null,
    sha256 text not null,
    source_instance text not null,
    created_at timestamptz not null default now(),
    metadata jsonb not null,
    unique (player_uuid, scope, revision),
    constraint player_profile_snapshots_sha_check check (sha256 ~ '^[0-9A-Fa-f]{64}$')
);

create table if not exists player_settings (
    player_uuid uuid primary key references player_identities(player_uuid),
    language text not null,
    menu_enabled boolean not null default true,
    hud_enabled boolean not null default true,
    tips_enabled boolean not null default true,
    privacy jsonb not null default '{}'::jsonb,
    updated_at timestamptz not null default now()
);

create index if not exists player_sessions_player_idx on player_sessions (player_uuid);
create index if not exists player_profile_snapshots_player_idx on player_profile_snapshots (player_uuid, scope);
