ALTER TABLE assets ADD COLUMN locked_claim_id uuid REFERENCES claims(id);
ALTER TABLE assets ADD COLUMN cancel_requested boolean NOT NULL DEFAULT false;
UPDATE assets SET locked_claim_id=claim_id
WHERE state IN ('capturing','quarantined') OR kind='land' AND state IN ('escrowed','listed');
CREATE UNIQUE INDEX assets_one_claim_lock ON assets(locked_claim_id) WHERE locked_claim_id IS NOT NULL;
ALTER TABLE assets ADD CONSTRAINT assets_physical_lock_required CHECK (
  kind='items' OR state NOT IN ('capturing','placing','quarantined') OR locked_claim_id IS NOT NULL
);
ALTER TABLE assets ADD CONSTRAINT assets_land_lock_required CHECK (
  kind<>'land' OR state NOT IN ('escrowed','listed') OR locked_claim_id=claim_id AND locked_claim_id IS NOT NULL
);
