alter table runtime_effect_workflows
    add column operation_id uuid,
    add column observation jsonb;

update runtime_effect_workflows set operation_id = id where operation_id is null;
alter table runtime_effect_workflows alter column operation_id set not null;
alter table runtime_effect_workflows add constraint runtime_effect_operation_unique unique (operation_id);
alter table runtime_effect_workflows drop constraint runtime_effect_workflows_instance_id_fkey;
alter table runtime_effect_workflows add constraint runtime_effect_workflows_instance_id_fkey
    foreign key (instance_id) references instances(id) on delete cascade;

create table runtime_instance_fences (
    instance_id text primary key references instances(id) on delete cascade,
    fence bigint not null check (fence > 0),
    operation_id uuid not null,
    correlation_id uuid not null,
    intent text not null check (intent in ('start','stop','observe','delete')),
    updated_at timestamptz not null default now()
);

create table runtime_reconcile_history (
    id bigserial primary key,
    instance_id text not null references instances(id) on delete cascade,
    operation_id uuid not null,
    correlation_id uuid not null,
    fence bigint not null check (fence > 0),
    attempt bigint not null check (attempt > 0),
    phase text not null check (phase in ('intent','ownership','effect','observation','outcome')),
    outcome text not null check (outcome in ('pending','succeeded','failed','unknown','stale','noop')),
    detail text,
    created_at timestamptz not null default now()
);

create index runtime_reconcile_history_instance_idx
    on runtime_reconcile_history (instance_id, id);
