create table if not exists player_pending_teleports (
    player_uuid uuid primary key references player_identities(player_uuid),
    target_server text not null,
    location jsonb not null,
    source text not null,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null default now() + interval '60 seconds'
);

create index if not exists player_pending_teleports_target_idx
    on player_pending_teleports (target_server, expires_at);
