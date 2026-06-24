create table if not exists points_accounts (
    player_uuid uuid primary key references player_identities(player_uuid),
    balance bigint not null default 0,
    updated_at timestamptz not null default now()
);

create table if not exists points_ledger (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    delta bigint not null,
    reason text not null,
    correlation_id uuid,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb
);

create table if not exists homes (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    name text not null,
    server_id text not null,
    location jsonb not null,
    created_at timestamptz not null default now(),
    unique (player_uuid, name)
);

create table if not exists warps (
    name text primary key,
    server_id text not null,
    location jsonb not null,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb
);

create table if not exists parties (
    id uuid primary key,
    owner_uuid uuid not null references player_identities(player_uuid),
    name text,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb
);

create table if not exists party_members (
    party_id uuid not null references parties(id) on delete cascade,
    player_uuid uuid not null references player_identities(player_uuid),
    role text not null,
    joined_at timestamptz not null default now(),
    primary key (party_id, player_uuid)
);

create table if not exists achievements (
    id text primary key,
    title_key text not null,
    config jsonb not null,
    created_at timestamptz not null default now()
);

create table if not exists player_achievements (
    player_uuid uuid not null references player_identities(player_uuid),
    achievement_id text not null references achievements(id),
    progress jsonb not null,
    claimed boolean not null default false,
    updated_at timestamptz not null default now(),
    primary key (player_uuid, achievement_id)
);

create index if not exists points_ledger_player_idx on points_ledger (player_uuid);
create index if not exists homes_player_idx on homes (player_uuid);
create index if not exists party_members_player_idx on party_members (player_uuid);
