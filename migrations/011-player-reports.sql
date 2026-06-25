create table if not exists player_reports (
    id uuid primary key,
    reporter_uuid uuid not null references player_identities(player_uuid),
    target_uuid uuid not null references player_identities(player_uuid),
    server_id text not null,
    reason text not null,
    status text not null default 'open',
    created_at timestamptz not null default now(),
    resolved_at timestamptz,
    constraint player_reports_status_check check (status in ('open', 'resolved', 'dismissed'))
);

create index if not exists player_reports_status_idx on player_reports (status, created_at desc);
create index if not exists player_reports_target_idx on player_reports (target_uuid, created_at desc);
