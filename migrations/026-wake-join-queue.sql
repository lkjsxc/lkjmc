create table if not exists wake_join_queue (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    player_name text not null,
    target_instance_id text not null references instances(id) on delete cascade,
    requested_by_kind text not null,
    requested_by_name text not null,
    state text not null,
    target_server text,
    failure_reason text,
    expires_at timestamptz not null,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint wake_join_queue_state_check check (
        state in ('queued', 'waking', 'ready', 'failed', 'expired')
    )
);

create index if not exists wake_join_queue_player_idx
    on wake_join_queue (player_uuid, created_at desc);

create index if not exists wake_join_queue_target_idx
    on wake_join_queue (target_instance_id, state, expires_at);
