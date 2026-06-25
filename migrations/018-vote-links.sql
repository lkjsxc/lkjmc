create table if not exists vote_links (
    id text primary key,
    title_key text not null,
    url text not null,
    sort_order integer not null default 0,
    updated_at timestamptz not null default now()
);
