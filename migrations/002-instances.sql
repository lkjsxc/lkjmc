create table if not exists instances (
    id text primary key,
    node_id uuid references nodes(id),
    kind text not null,
    desired_state text not null,
    jar_asset_id uuid,
    template_id text,
    config jsonb not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint instances_kind_check check (
        kind in ('velocity', 'paper', 'folia', 'vanilla-custom', 'modded-custom')
    ),
    constraint instances_desired_state_check check (
        desired_state in ('stopped', 'starting', 'running', 'stopping', 'restarting', 'deleting', 'failed')
    )
);

create table if not exists instance_observations (
    instance_id text primary key references instances(id) on delete cascade,
    observed_state text not null,
    pid integer,
    healthy boolean not null,
    started_at timestamptz,
    updated_at timestamptz not null default now(),
    message text,
    constraint instance_observed_state_check check (
        observed_state in ('process-absent', 'process-starting', 'process-healthy',
        'process-unhealthy', 'process-exited', 'process-unknown')
    )
);

create table if not exists instance_events (
    id uuid primary key,
    instance_id text not null references instances(id) on delete cascade,
    event_type text not null,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb
);

create table if not exists instance_ports (
    port integer primary key,
    instance_id text not null references instances(id) on delete cascade,
    purpose text not null,
    created_at timestamptz not null default now(),
    constraint instance_ports_range_check check (port > 0 and port <= 65535)
);

create table if not exists templates (
    id text primary key,
    kind text not null,
    config jsonb not null,
    created_at timestamptz not null default now()
);

create index if not exists instances_node_id_idx on instances (node_id);
create index if not exists instance_events_instance_id_idx on instance_events (instance_id);
create index if not exists instance_ports_instance_id_idx on instance_ports (instance_id);
