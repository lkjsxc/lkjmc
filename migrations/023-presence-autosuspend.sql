alter table instances drop constraint if exists instances_kind_check;
alter table instances add constraint instances_kind_check check (
    kind in ('velocity', 'paper', 'folia', 'purpur', 'vanilla-custom', 'modded-custom')
);

alter table instances drop constraint if exists instances_desired_state_check;
alter table instances add constraint instances_desired_state_check check (
    desired_state in ('stopped', 'starting', 'running', 'suspended', 'stopping',
    'restarting', 'deleting', 'failed')
);

create table if not exists instance_presence (
    instance_id text primary key references instances(id) on delete cascade,
    last_heartbeat_at timestamptz not null,
    player_count integer check (player_count is null or player_count >= 0),
    max_players integer,
    ready boolean not null default false,
    empty_since timestamptz,
    last_nonempty_at timestamptz,
    last_suspend_at timestamptz,
    last_wake_at timestamptz,
    suspend_reason text,
    metadata jsonb not null default '{}'::jsonb,
    updated_at timestamptz not null default now()
);

create index if not exists instance_presence_empty_idx on instance_presence (empty_since);
