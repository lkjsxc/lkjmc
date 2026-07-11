alter table shop_purchases
    add column if not exists correlation_id uuid;

create unique index if not exists shop_purchases_correlation_idx
    on shop_purchases (correlation_id) where correlation_id is not null;

create or replace function points_balance_matches_ledger() returns trigger as $$
declare
    account uuid := coalesce(new.player_uuid, old.player_uuid);
    expected bigint;
    actual bigint;
begin
    select balance into actual from points_accounts where player_uuid = account;
    select coalesce(sum(delta), 0) into expected from points_ledger where player_uuid = account;
    if actual is distinct from expected then
        raise exception 'points balance does not match ledger for %', account;
    end if;
    return null;
end;
$$ language plpgsql;

drop trigger if exists points_ledger_balance_check on points_ledger;
create constraint trigger points_ledger_balance_check
    after insert or update or delete on points_ledger
    deferrable initially deferred for each row
    execute function points_balance_matches_ledger();
