create table if not exists player_punishments (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    player_name text not null,
    kind text not null,
    reason text not null,
    actor_name text not null,
    created_at timestamptz not null default now(),
    expires_at timestamptz,
    revoked_at timestamptz,
    constraint player_punishments_kind_check check (kind in ('ban'))
);

create index if not exists player_punishments_active_idx
    on player_punishments (player_uuid, kind, revoked_at, expires_at);
