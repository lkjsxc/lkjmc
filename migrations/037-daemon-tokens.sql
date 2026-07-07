create table if not exists daemon_tokens (
    credential_id uuid primary key,
    token_hash text not null unique,
    surface text not null,
    scopes text[] not null default '{}',
    created_at timestamptz not null default now(),
    last_used_at timestamptz,
    expires_at timestamptz,
    revoked_at timestamptz,
    constraint daemon_tokens_surface_check check (
        surface in ('cli', 'web', 'paper', 'velocity', 'discord', 'installer', 'daemon')
    )
);

create index if not exists daemon_tokens_active_idx
    on daemon_tokens (token_hash)
    where revoked_at is null;
