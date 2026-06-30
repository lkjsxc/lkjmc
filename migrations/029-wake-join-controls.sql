alter table wake_join_queue
    add column if not exists consumed_at timestamptz,
    add column if not exists cancelled_at timestamptz,
    add column if not exists cleanup_after timestamptz,
    add column if not exists correlation_id text;

alter table wake_join_queue drop constraint if exists wake_join_queue_state_check;

alter table wake_join_queue add constraint wake_join_queue_state_check check (
    state in ('queued', 'starting', 'ready', 'transferred', 'failed',
              'cancelled', 'expired', 'denied')
);

create index if not exists wake_join_queue_live_idx
    on wake_join_queue (player_uuid, target_instance_id, state, expires_at)
    where state in ('queued', 'starting', 'ready');
