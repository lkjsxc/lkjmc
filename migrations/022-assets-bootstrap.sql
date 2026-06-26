create table if not exists assets (
    id uuid primary key,
    asset_kind text not null,
    platform text not null,
    project text not null,
    channel text not null,
    name text not null,
    file_name text not null,
    path text not null unique,
    sha256 text not null,
    size_bytes bigint not null,
    source text not null,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);

create table if not exists asset_downloads (
    id uuid primary key,
    asset_id uuid references assets(id) on delete set null,
    asset_kind text not null,
    project text not null,
    channel text not null,
    url text not null,
    result text not null,
    sha256 text,
    size_bytes bigint,
    error text,
    created_at timestamptz not null default now()
);

create table if not exists plugin_catalog_entries (
    plugin_id text primary key,
    display_name text not null,
    platforms jsonb not null,
    default_policy text not null,
    source_kind text not null,
    source_project text not null,
    required_plugin_ids jsonb not null default '[]'::jsonb,
    metadata jsonb not null default '{}'::jsonb,
    updated_at timestamptz not null default now()
);

create table if not exists instance_plugin_installations (
    instance_id text not null references instances(id) on delete cascade,
    plugin_id text not null,
    asset_id uuid not null references assets(id),
    target_path text not null,
    installed_sha256 text not null,
    installed_at timestamptz not null default now(),
    primary key (instance_id, plugin_id)
);

create table if not exists bootstrap_runs (
    id uuid primary key,
    profile text not null,
    requested_by text not null,
    result text not null,
    diagnostics jsonb not null,
    started_at timestamptz not null default now(),
    finished_at timestamptz
);

create table if not exists bootstrap_steps (
    id uuid primary key,
    run_id uuid not null references bootstrap_runs(id) on delete cascade,
    step_order integer not null,
    effect_kind text not null,
    target text not null,
    result text not null,
    diagnostic text,
    created_at timestamptz not null default now()
);

insert into assets
(id, asset_kind, platform, project, channel, name, file_name, path, sha256,
 size_bytes, source, metadata, created_at)
select id, 'server', kind, project, channel, name, name, path, sha256,
       size_bytes, source, metadata, created_at
from jar_assets
on conflict (path) do nothing;

create index if not exists assets_lookup_idx on assets (asset_kind, platform, project, channel);
create index if not exists asset_downloads_lookup_idx on asset_downloads (asset_kind, project, channel);
create index if not exists bootstrap_steps_run_idx on bootstrap_steps (run_id, step_order);
