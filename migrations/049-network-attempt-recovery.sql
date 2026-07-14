alter table network_apply_attempts
    drop constraint network_apply_attempts_outcome_check;

alter table network_apply_attempts
    add constraint network_apply_attempts_outcome_check check (outcome in
        ('planned', 'applying', 'observed', 'failed', 'unknown', 'unsupported', 'no-op'));

alter table network_apply_attempts
    add column effect_phase text not null default 'none' check (effect_phase in
        ('none', 'configuration', 'runtime', 'observation'));
