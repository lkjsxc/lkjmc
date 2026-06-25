create table if not exists announcements (
    id uuid primary key,
    actor_name text not null,
    server_id text not null,
    message text not null,
    created_at timestamptz not null default now()
);

create index if not exists announcements_server_idx
    on announcements (server_id, created_at desc);
