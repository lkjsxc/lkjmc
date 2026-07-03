create table if not exists link_codes (
    id uuid primary key,
    player_uuid uuid not null,
    player_name text not null,
    code_hash text not null unique,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null,
    consumed_at timestamptz
);

create unique index if not exists link_codes_active_player_idx
    on link_codes (player_uuid)
    where consumed_at is null;
