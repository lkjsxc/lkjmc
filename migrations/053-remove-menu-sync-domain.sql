drop trigger if exists sync_menus_shop on shop_items;
drop trigger if exists sync_menus_kits on kit_definitions;
drop trigger if exists sync_menus_votes on vote_links;
drop trigger if exists sync_menus_plugins on plugin_catalog_entries;

delete from sync_change_feed where domain = 'menus';
delete from sync_change_archive where domain = 'menus';
delete from sync_domain_revisions where domain = 'menus';

alter table sync_domain_revisions
    drop constraint if exists sync_domain_revisions_domain_check;
alter table sync_domain_revisions
    add constraint sync_domain_revisions_domain_check check (domain in
      ('permissions','claims','profiles','presence','routing','settings'));
