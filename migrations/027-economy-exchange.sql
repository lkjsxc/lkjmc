create table if not exists economy_exchange_rates (
    id text primary key,
    material text not null unique,
    title_key text not null,
    category text not null,
    points_per_item bigint not null,
    min_amount bigint not null default 1,
    enabled boolean not null default true,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint economy_exchange_points_check check (points_per_item >= 0),
    constraint economy_exchange_amount_check check (min_amount > 0)
);

create table if not exists economy_exchange_events (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    rate_id text not null references economy_exchange_rates(id),
    material text not null,
    amount bigint not null,
    points_delta bigint not null,
    ledger_id uuid not null references points_ledger(id),
    correlation_id uuid not null unique,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    constraint economy_exchange_amount_positive check (amount > 0),
    constraint economy_exchange_points_nonnegative check (points_delta >= 0)
);

create unique index if not exists points_ledger_correlation_idx
    on points_ledger (correlation_id) where correlation_id is not null;
create index if not exists economy_exchange_events_player_idx
    on economy_exchange_events (player_uuid, created_at desc);
