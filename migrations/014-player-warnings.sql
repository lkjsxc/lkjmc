create table if not exists player_warnings (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    player_name text not null,
    actor_name text not null,
    reason text not null,
    created_at timestamptz not null default now()
);

create index if not exists player_warnings_player_idx
    on player_warnings (player_uuid, created_at desc);
