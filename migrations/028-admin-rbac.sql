create table if not exists admin_roles (
    id text primary key,
    title_key text not null,
    permissions jsonb not null,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);

create table if not exists admin_grants (
    id uuid primary key,
    principal_kind text not null,
    principal_id text not null,
    role_id text not null references admin_roles(id),
    scope text not null default 'global',
    expires_at timestamptz,
    reason text not null,
    granted_by_kind text not null,
    granted_by_id text not null,
    revoked_at timestamptz,
    revoked_by_kind text,
    revoked_by_id text,
    revoke_reason text,
    created_at timestamptz not null default now()
);

create table if not exists admin_audit (
    id uuid primary key,
    actor_kind text not null,
    actor_id text not null,
    subject_kind text,
    subject_id text,
    action text not null,
    target_kind text,
    target_id text,
    result text not null,
    correlation_id uuid,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);

create index if not exists admin_grants_principal_idx
    on admin_grants (principal_kind, principal_id) where revoked_at is null;
create index if not exists admin_audit_created_idx on admin_audit (created_at desc);
