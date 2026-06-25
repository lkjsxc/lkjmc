create table if not exists player_notes (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    player_name text not null,
    actor_name text not null,
    body text not null,
    created_at timestamptz not null default now()
);

create index if not exists player_notes_player_idx
    on player_notes (player_uuid, created_at desc);
