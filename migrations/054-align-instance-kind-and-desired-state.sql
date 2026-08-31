alter table instances
    drop constraint if exists instances_kind_check;

alter table instances
    add constraint instances_kind_check check (
        kind in (
            'velocity', 'paper', 'folia', 'purpur',
            'vanilla-custom', 'modded-custom'
        )
    );

alter table instances
    drop constraint if exists instances_desired_state_check;

alter table instances
    add constraint instances_desired_state_check check (
        desired_state in (
            'stopped', 'starting', 'running', 'suspended',
            'stopping', 'restarting', 'deleting', 'failed'
        )
    );
