ALTER TABLE servers ADD COLUMN maintenance_job_id uuid REFERENCES jobs;
ALTER TABLE jobs ADD COLUMN host_authorized_at timestamptz;
