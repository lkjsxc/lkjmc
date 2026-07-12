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
    if TG_OP = 'UPDATE'
       and OLD.token_hash is not distinct from NEW.token_hash
       and OLD.surface is not distinct from NEW.surface
       and OLD.principal_kind is not distinct from NEW.principal_kind
       and OLD.principal_id is not distinct from NEW.principal_id
       and OLD.scopes is not distinct from NEW.scopes
       and OLD.expires_at is not distinct from NEW.expires_at
       and OLD.revoked_at is not distinct from NEW.revoked_at then
        return null;
    end if;
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
after insert or delete or update of token_hash, surface, principal_kind,
    principal_id, scopes, expires_at, revoked_at on daemon_tokens
for each row execute function bump_daemon_token_revision();
