create table if not exists network_intents (
    revision bigserial primary key,
    authored_revision bigint not null check (authored_revision > 0),
    intent_digest text not null check (intent_digest ~ '^[0-9a-f]{64}$'),
    intent jsonb not null check (jsonb_typeof(intent) = 'object'),
    correlation text not null unique check (length(correlation) between 1 and 128),
    created_at timestamptz not null default now()
);

create table if not exists network_apply_attempts (
    id uuid primary key,
    network_revision bigint not null references network_intents(revision),
    correlation text not null check (length(correlation) between 1 and 128),
    outcome text not null check (outcome in
        ('planned', 'applying', 'observed', 'failed', 'unsupported', 'no-op')),
    diagnostic text,
    observation jsonb not null default '{}'::jsonb,
    started_at timestamptz not null default now(),
    finished_at timestamptz,
    check ((outcome in ('planned', 'applying')) = (finished_at is null))
);

create index if not exists network_attempt_revision_idx
    on network_apply_attempts (network_revision, started_at desc);
