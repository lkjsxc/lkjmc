CREATE TABLE official_backup_steps (
    backup_id uuid PRIMARY KEY REFERENCES backups,
    job_id uuid NOT NULL UNIQUE REFERENCES jobs,
    phase text NOT NULL CHECK(phase IN ('frozen','dumping','dumped','released')),
    database_manifest jsonb,
    world_manifest jsonb,
    error text,
    frozen_at timestamptz NOT NULL DEFAULT now(),
    released_at timestamptz
);
INSERT INTO settings(key,value) VALUES('official_backup_owner','null');
