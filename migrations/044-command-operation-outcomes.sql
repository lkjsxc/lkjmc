alter table commands drop constraint commands_result_check;

alter table commands alter column id type text using id::text;
alter table commands add column completed_at timestamptz;
update commands set completed_at = created_at where result <> 'requested';

alter table commands add constraint commands_result_check
check (result in ('requested', 'succeeded', 'failed', 'denied', 'cancelled'));
alter table commands add constraint commands_completion_check
check ((result = 'requested') = (completed_at is null));
