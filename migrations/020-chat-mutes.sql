alter table player_punishments
    drop constraint if exists player_punishments_kind_check;

alter table player_punishments
    add constraint player_punishments_kind_check check (kind in ('ban', 'mute'));
