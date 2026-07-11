alter table shop_purchases
    add column if not exists correlation_id uuid;

create unique index if not exists shop_purchases_correlation_idx
    on shop_purchases (correlation_id) where correlation_id is not null;
