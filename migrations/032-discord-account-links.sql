create table if not exists discord_account_links (
    discord_user_id text primary key,
    minecraft_uuid uuid not null references player_identities(player_uuid),
    verification_state text not null,
    created_at timestamptz not null default now(),
    verified_at timestamptz,
    revoked_at timestamptz,
    metadata jsonb not null default '{}'::jsonb
);

create index if not exists discord_account_links_minecraft_idx
    on discord_account_links (minecraft_uuid)
    where revoked_at is null;
