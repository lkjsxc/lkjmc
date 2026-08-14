alter table daemon_tokens
    drop constraint if exists daemon_tokens_principal_kind_check,
    add constraint daemon_tokens_principal_kind_check check (
        principal_kind in ('minecraft-player', 'discord-user', 'operator', 'service', 'instance')
    );
