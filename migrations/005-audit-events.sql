create table if not exists commands (
    id uuid primary key,
    actor_kind text not null,
    actor_name text not null,
    command text not null,
    body jsonb not null,
    result text not null,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb,
    constraint commands_result_check check (result in ('requested', 'succeeded', 'failed', 'denied'))
);

create table if not exists audit_events (
    id uuid primary key,
    actor_kind text not null,
    actor_name text not null,
    action text not null,
    target_kind text not null,
    target_id text not null,
    result text not null,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb,
    constraint audit_events_result_check check (result in ('requested', 'succeeded', 'failed', 'denied'))
);

create table if not exists outbox_events (
    id uuid primary key,
    topic text not null,
    payload jsonb not null,
    created_at timestamptz not null default now(),
    dispatched_at timestamptz,
    metadata jsonb not null default '{}'::jsonb
);

create index if not exists commands_actor_idx on commands (actor_kind, actor_name);
create index if not exists audit_events_target_idx on audit_events (target_kind, target_id);
create index if not exists outbox_events_pending_idx on outbox_events (dispatched_at) where dispatched_at is null;
