-- Isolated E-DATA candidate schema; __SCHEMA__ is a disposable identifier.
create schema __SCHEMA__;
create extension if not exists pgcrypto with schema __SCHEMA__;

create function __SCHEMA__.profile_sha(payload jsonb) returns text
language sql immutable as $$
    select encode(__SCHEMA__.digest(convert_to(payload::text, 'utf8'), 'sha256'), 'hex')
$$;

create table __SCHEMA__.opaque_profiles (
    id uuid primary key,
    payload bytea not null
);

create function __SCHEMA__.valid_profile(payload jsonb) returns boolean
language plpgsql immutable as $$
declare item jsonb;
begin
    if jsonb_typeof(payload) <> 'object'
       or payload->>'formatVersion' <> '1'
       or jsonb_typeof(payload->'items') <> 'array'
       or jsonb_array_length(payload->'items') > 36
       or coalesce(payload->>'selectedSlot', '') !~ '^[0-8]$' then
        return false;
    end if;
    for item in select value from jsonb_array_elements(payload->'items') loop
        if jsonb_typeof(item) <> 'object'
           or jsonb_typeof(item->'kind') <> 'string'
           or jsonb_typeof(item->'data') <> 'string' then
            return false;
        end if;
    end loop;
    return true;
end $$;

create table __SCHEMA__.typed_profiles (
    id uuid primary key,
    revision bigint not null check (revision > 0),
    payload jsonb not null check (__SCHEMA__.valid_profile(payload)),
    sha256 text not null check (sha256 = __SCHEMA__.profile_sha(payload)),
    unique (id, revision)
);

create table __SCHEMA__.weak_deliveries (
    correlation uuid not null,
    state text not null
);

create table __SCHEMA__.fenced_deliveries (
    correlation uuid primary key,
    state text not null check (state in ('pending', 'claimed', 'acknowledged')),
    holder text,
    fence bigint not null default 0 check (fence >= 0),
    claimed_at timestamptz,
    acknowledged_at timestamptz
);

create table __SCHEMA__.delivery_compensations (
    correlation uuid primary key,
    reason text not null,
    created_at timestamptz not null default now()
);

create table __SCHEMA__.profile_heads (
    player_uuid uuid primary key,
    revision bigint not null check (revision >= 0)
);

create table __SCHEMA__.profile_events (
    player_uuid uuid not null references __SCHEMA__.profile_heads(player_uuid),
    revision bigint not null,
    snapshot jsonb not null,
    created_at timestamptz not null default now(),
    primary key (player_uuid, revision)
);
