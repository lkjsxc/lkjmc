alter table daemon_tokens
    add column if not exists principal_kind text,
    add column if not exists principal_id text;

update daemon_tokens
set surface = 'daemon',
    principal_kind = 'service',
    principal_id = 'migrated-' || credential_id::text,
    scopes = array['lkjmc.migrated.none']::text[],
    expires_at = least(coalesce(expires_at, created_at + interval '24 hours'), now() + interval '24 hours');

alter table daemon_tokens
    alter column principal_kind set not null,
    alter column principal_id set not null,
    alter column expires_at set not null,
    drop constraint if exists daemon_tokens_surface_check,
    add constraint daemon_tokens_surface_check check (surface in ('cli', 'web', 'paper', 'velocity', 'installer', 'daemon')),
    add constraint daemon_tokens_principal_kind_check check (principal_kind in ('minecraft-player', 'discord-user', 'operator', 'service')),
    add constraint daemon_tokens_principal_id_check check (length(principal_id) between 1 and 200),
    add constraint daemon_tokens_scopes_check check (cardinality(scopes) > 0);

create index if not exists daemon_tokens_principal_active_idx
    on daemon_tokens (surface, principal_kind, principal_id)
    where revoked_at is null;
