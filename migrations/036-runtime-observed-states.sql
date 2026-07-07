alter table instance_observations
    drop constraint if exists instance_observed_state_check;

alter table instance_observations
    add constraint instance_observed_state_check check (
        observed_state in (
            'process-absent', 'process-starting', 'process-healthy',
            'process-unhealthy', 'process-exited', 'process-unknown',
            'kubernetes-absent', 'kubernetes-starting', 'kubernetes-ready',
            'kubernetes-unhealthy', 'kubernetes-exited', 'kubernetes-unknown',
            'runtime-absent', 'runtime-starting', 'runtime-ready',
            'runtime-unhealthy', 'runtime-exited', 'runtime-unknown'
        )
    );
