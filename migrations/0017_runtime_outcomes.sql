-- Console delivery can be unknowable after the FIFO write. Never replay it.
ALTER TABLE jobs DROP CONSTRAINT jobs_state_check;
ALTER TABLE jobs ADD CONSTRAINT jobs_state_check
  CHECK (state IN ('queued','leased','waiting','succeeded','failed','cancelled','delivery_unknown'));
ALTER TABLE jobs ADD CONSTRAINT console_unknown_only
  CHECK (state <> 'delivery_unknown' OR kind = 'server.console');

-- Host machine observations never substitute for Minecraft readiness.
ALTER TABLE servers ADD COLUMN machine_observed text NOT NULL DEFAULT 'unknown'
  CHECK (machine_observed IN ('unknown','running','stopped'));
ALTER TABLE servers ADD COLUMN machine_observed_at timestamptz;
