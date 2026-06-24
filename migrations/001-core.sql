create table if not exists schema_migrations (
    version integer primary key,
    name text not null,
    applied_at timestamptz not null default now()
);

create table if not exists nodes (
    id uuid primary key,
    name text unique not null,
    hostname text not null,
    kind text not null,
    created_at timestamptz not null default now(),
    last_seen_at timestamptz,
    metadata jsonb not null default '{}'::jsonb,
    constraint nodes_kind_check check (kind in ('local-process', 'test'))
);

create index if not exists nodes_hostname_idx on nodes (hostname);
