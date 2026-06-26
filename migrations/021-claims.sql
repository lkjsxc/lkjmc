create table player_claims (
    id uuid primary key,
    owner_uuid uuid not null,
    owner_name text not null,
    name text not null,
    name_key text not null,
    created_at timestamptz not null default now(),
    deleted_at timestamptz
);

create unique index player_claims_active_owner_name_key
    on player_claims(owner_uuid, name_key)
    where deleted_at is null;

create table claim_chunks (
    claim_id uuid not null references player_claims(id) on delete cascade,
    instance_id text not null,
    world_name text not null,
    chunk_x integer not null,
    chunk_z integer not null,
    primary key (claim_id, instance_id, world_name, chunk_x, chunk_z)
);

create unique index claim_chunks_unique_active_chunk
    on claim_chunks(instance_id, world_name, chunk_x, chunk_z);

create table claim_trusts (
    claim_id uuid not null references player_claims(id) on delete cascade,
    trusted_uuid uuid not null,
    trusted_name text not null,
    created_at timestamptz not null default now(),
    primary key (claim_id, trusted_uuid)
);
