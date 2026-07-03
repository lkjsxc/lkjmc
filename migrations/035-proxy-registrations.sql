create table if not exists proxy_registrations (
    instance_id text primary key references instances(id) on delete cascade,
    connect_host text not null,
    connect_port integer not null,
    registered boolean not null,
    failure_reason text,
    reported_at timestamptz not null default now()
);

create index if not exists proxy_registrations_reported_idx
    on proxy_registrations (reported_at desc);
