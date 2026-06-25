create table if not exists player_daily_claims (
    player_uuid uuid not null references player_identities(player_uuid),
    claim_date date not null default current_date,
    points bigint not null,
    created_at timestamptz not null default now(),
    primary key (player_uuid, claim_date),
    constraint player_daily_claims_points_check check (points > 0)
);
