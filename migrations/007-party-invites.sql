create table if not exists party_invites (
    id uuid primary key,
    party_id uuid not null references parties(id) on delete cascade,
    inviter_uuid uuid not null references player_identities(player_uuid),
    invitee_uuid uuid not null references player_identities(player_uuid),
    expires_at timestamptz not null,
    accepted_at timestamptz,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb
);

create index if not exists party_invites_invitee_idx on party_invites (invitee_uuid, expires_at);
