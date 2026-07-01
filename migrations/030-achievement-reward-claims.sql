alter table player_achievements
    add column if not exists reward_claimed boolean not null default false;

alter table player_achievements
    add column if not exists reward_claimed_at timestamptz;

update player_achievements
set reward_claimed = true,
    reward_claimed_at = coalesce(reward_claimed_at, updated_at)
where claimed = true and reward_claimed = false;

create table if not exists achievement_reward_claims (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid) on delete cascade,
    achievement_id text not null references achievements(id) on delete cascade,
    reward_id text not null,
    reward_kind text not null,
    points_delta bigint not null default 0,
    ledger_id uuid references points_ledger(id) on delete set null,
    claimed_at timestamptz not null default now(),
    metadata jsonb not null default '{}'::jsonb,
    unique (player_uuid, achievement_id, reward_id)
);
