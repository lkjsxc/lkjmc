ALTER TABLE backups DROP CONSTRAINT backups_state_check;
ALTER TABLE backups ADD CONSTRAINT backups_state_check CHECK(state IN
    ('queued','freezing','saving','verifying','ready','failed','restoring','pruning','pruned'));
ALTER TABLE backups ADD COLUMN job_id uuid REFERENCES jobs;
ALTER TABLE backups ADD COLUMN scheduled_for date;
ALTER TABLE backups ADD COLUMN completed_at timestamptz;
ALTER TABLE backups ADD COLUMN pinned boolean NOT NULL DEFAULT false;
ALTER TABLE backups ADD COLUMN prune_job_id uuid REFERENCES jobs;
ALTER TABLE backups ADD COLUMN database_prune_started_at timestamptz;
ALTER TABLE backups ADD COLUMN database_pruned_at timestamptz;
ALTER TABLE backups ADD COLUMN pruned_at timestamptz;
ALTER TABLE backups ADD CONSTRAINT scheduled_official_only CHECK(scheduled_for IS NULL OR kind='official');
UPDATE backups b SET job_id=j.id FROM jobs j
    WHERE j.kind IN ('official.backup','server.backup') AND j.payload->>'backup_id'=b.id::text;
UPDATE backups b SET completed_at=coalesce((SELECT updated_at FROM jobs WHERE id=b.job_id),b.created_at)
    WHERE b.state='ready';
CREATE UNIQUE INDEX one_scheduled_backup_per_day ON backups(server_id,scheduled_for)
    WHERE scheduled_for IS NOT NULL;
CREATE UNIQUE INDEX one_official_backup_in_progress ON backups(kind)
    WHERE kind='official' AND state IN ('queued','freezing','saving','verifying','restoring');
