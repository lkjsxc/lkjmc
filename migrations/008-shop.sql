create table if not exists shop_items (
    id text primary key,
    title_key text not null,
    price_points bigint not null,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    constraint shop_items_price_check check (price_points >= 0)
);

create table if not exists shop_purchases (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    item_id text not null references shop_items(id),
    price_points bigint not null,
    created_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb
);

create index if not exists shop_purchases_player_idx on shop_purchases (player_uuid);
