create table if not exists daemon_token_revision (
    singleton boolean primary key default true check (singleton),
    revision bigint not null check (revision > 0)
);

insert into daemon_token_revision (singleton, revision)
values (true, 1)
on conflict (singleton) do nothing;

create or replace function bump_daemon_token_revision()
returns trigger
language plpgsql
as $$
declare
    next_revision bigint;
begin
    update daemon_token_revision
    set revision = revision + 1
    where singleton = true
    returning revision into next_revision;
    perform pg_notify('lkjmc_daemon_token_revision', next_revision::text);
    return null;
end;
$$;

drop trigger if exists daemon_token_revision_changed on daemon_tokens;
create trigger daemon_token_revision_changed
after insert or update or delete on daemon_tokens
for each statement execute function bump_daemon_token_revision();
