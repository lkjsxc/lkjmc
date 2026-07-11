do $$
declare
    bad_item text;
begin
    select id into bad_item
    from shop_items
    where metadata #>> '{delivery,executor}' in ('adventure', 'adventure-end-expedition')
      and not (
        id = 'adventure-end-expedition'
        and metadata in (
            '{"delivery":{"executor":"adventure","adventureId":"end-expedition"}}'::jsonb,
            '{"delivery":{"executor":"adventure-end-expedition"}}'::jsonb,
            '{"category":"adventures","delivery":{"executor":"adventure","adventureId":"end-expedition"}}'::jsonb
        )
      )
    limit 1;
    if bad_item is not null then
        raise exception 'migration 042 cannot canonicalize shop item %: use only adventure-end-expedition with documented canonical metadata', bad_item;
    end if;
end $$;

update shop_items
set metadata = '{"delivery":{"executor":"adventure","adventureId":"end-expedition"}}'::jsonb
where id = 'adventure-end-expedition'
  and metadata in (
      '{"delivery":{"executor":"adventure","adventureId":"end-expedition"}}'::jsonb,
      '{"delivery":{"executor":"adventure-end-expedition"}}'::jsonb,
      '{"category":"adventures","delivery":{"executor":"adventure","adventureId":"end-expedition"}}'::jsonb
  );

alter table shop_items
    drop constraint if exists shop_items_adventure_delivery_check,
    add constraint shop_items_adventure_delivery_check check (
        (id = 'adventure-end-expedition'
         and metadata = '{"delivery":{"executor":"adventure","adventureId":"end-expedition"}}'::jsonb)
        or
        (id <> 'adventure-end-expedition'
         and coalesce(metadata #>> '{delivery,executor}', '')
             not in ('adventure', 'adventure-end-expedition'))
    );
