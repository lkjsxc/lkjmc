create table if not exists player_vote_rewards (
    id uuid primary key,
    player_uuid uuid not null references player_identities(player_uuid),
    player_name text not null,
    link_id text not null references vote_links(id),
    reward_points bigint not null,
    source text not null,
    created_at timestamptz not null default now()
);

create index if not exists player_vote_rewards_player_idx
    on player_vote_rewards (player_uuid, created_at desc);
