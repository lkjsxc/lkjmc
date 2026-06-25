create table if not exists kit_definitions (
    id text primary key,
    title_key text not null,
    reward_points bigint not null,
    cooldown_hours integer not null,
    metadata jsonb not null default '{}'::jsonb,
    updated_at timestamptz not null default now()
);

create table if not exists player_kit_claims (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    kit_id text not null references kit_definitions(id),
    reward_points bigint not null,
    claimed_at timestamptz not null default now()
);

create index if not exists player_kit_claims_player_idx
    on player_kit_claims (player_uuid, kit_id, claimed_at desc);
