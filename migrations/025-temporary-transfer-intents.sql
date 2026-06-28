create table if not exists temporary_transfer_intents (
    id uuid primary key,
    temporary_instance_id text not null
        references temporary_instances(instance_id) on delete cascade,
    player_uuid uuid not null,
    player_name text not null,
    state text not null,
    expires_at timestamptz not null,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint temporary_transfer_intents_state_check
        check (state in ('queued', 'completed', 'expired', 'cancelled', 'failed'))
);

create index if not exists temporary_transfer_intents_player_idx
    on temporary_transfer_intents (player_uuid, created_at desc);

create index if not exists temporary_transfer_intents_instance_idx
    on temporary_transfer_intents (temporary_instance_id, expires_at);
