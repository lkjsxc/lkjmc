create table if not exists jar_assets (
    id uuid primary key,
    kind text not null,
    project text not null,
    channel text not null,
    name text not null,
    path text not null unique,
    sha256 text not null,
    size_bytes bigint not null,
    source text not null,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb,
    constraint jar_kind_check check (
        kind in ('paper', 'folia', 'velocity', 'custom', 'vanilla-custom', 'modded-custom')
    ),
    constraint jar_sha256_check check (sha256 ~ '^[0-9A-Fa-f]{64}$'),
    constraint jar_size_check check (size_bytes >= 0)
);

create table if not exists jar_downloads (
    id uuid primary key,
    jar_asset_id uuid references jar_assets(id),
    project text not null,
    channel text not null,
    url text not null,
    result text not null,
    sha256 text,
    size_bytes bigint,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb,
    constraint jar_download_result_check check (result in ('requested', 'succeeded', 'failed'))
);

alter table instances
    add constraint instances_jar_asset_id_fk
    foreign key (jar_asset_id) references jar_assets(id);

create index if not exists jar_assets_kind_project_idx on jar_assets (kind, project);
create index if not exists jar_downloads_asset_idx on jar_downloads (jar_asset_id);
