create table observability_operations (
    operation_id uuid primary key,
    request_id text not null unique check (length(request_id) between 1 and 128),
    correlation_id uuid not null,
    command text not null check (length(command) between 1 and 96),
    actor_kind text not null check (length(actor_kind) between 1 and 32),
    actor_name text not null check (length(actor_name) between 1 and 96),
    surface text not null check (surface in ('cli','web','discord','paper','velocity','http','unix','internal')),
    outcome text not null check (outcome in ('succeeded','failed','denied','cancelled','degraded')),
    error_class text check (error_class is null or length(error_class) between 1 and 64),
    started_at timestamptz not null default now(),
    completed_at timestamptz not null default now()
);

create table observability_events (
    event_id uuid primary key,
    occurred_at timestamptz not null default now(),
    severity text not null check (severity in ('debug','info','warn','error')),
    component text not null check (component in ('daemon','cli','web','discord','runtime','network','sync','jvm')),
    event_kind text not null check (event_kind in ('command_completed','admission_rejected','database_diagnostic','runtime_diagnostic','network_diagnostic','sync_diagnostic','jvm_diagnostic','discord_diagnostic','readiness_checked','support_bundle_created')),
    request_id text check (request_id is null or length(request_id) between 1 and 128),
    operation_id uuid,
    correlation_id uuid,
    actor_kind text not null check (length(actor_kind) between 1 and 32),
    actor_name text not null check (length(actor_name) between 1 and 96),
    surface text not null check (surface in ('cli','web','discord','paper','velocity','http','unix','internal')),
    outcome text not null check (outcome in ('succeeded','failed','denied','cancelled','degraded')),
    error_class text check (error_class is null or length(error_class) between 1 and 64),
    attributes jsonb not null default '{}'::jsonb check (
        jsonb_typeof(attributes) = 'object'
        and attributes - array['command','serverId','route','runtime','fault','queue','reason','migration','retention','bundle','transport','source'] = '{}'::jsonb
        and pg_column_size(attributes) <= 4096
    ),
    source text not null check (length(source) between 1 and 64)
);

create index observability_events_request on observability_events (request_id, occurred_at desc);
create index observability_events_operation on observability_events (operation_id, occurred_at desc);
create index observability_events_correlation on observability_events (correlation_id, occurred_at desc);
create index observability_events_retention on observability_events (occurred_at);
