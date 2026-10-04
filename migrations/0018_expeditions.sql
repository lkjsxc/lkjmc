-- Participant records, rather than mutable party membership, own admission.
-- Existing records are the authoritative roster for already prepared runs.
ALTER TABLE adventure_participants ADD COLUMN consented_at timestamptz;
ALTER TABLE adventure_participants ADD COLUMN committed_at timestamptz;
UPDATE adventure_participants ap SET consented_at=a.created_at, committed_at=a.created_at
FROM adventures a WHERE a.id=ap.adventure_id;
ALTER TABLE adventure_participants ALTER COLUMN consented_at SET DEFAULT now();
ALTER TABLE adventure_participants ALTER COLUMN consented_at SET NOT NULL;
ALTER TABLE adventure_participants ALTER COLUMN committed_at SET DEFAULT now();
ALTER TABLE adventure_participants ALTER COLUMN committed_at SET NOT NULL;

-- Lifetime, terrain and access are independent. Legacy world IDs/names remain
-- stable because the physical journals and receipts prove ownership by those IDs.
ALTER TABLE worlds ADD COLUMN lifetime text NOT NULL DEFAULT 'persistent'
    CHECK(lifetime IN ('persistent','temporary'));
ALTER TABLE worlds ADD COLUMN environment text NOT NULL DEFAULT 'overworld'
    CHECK(environment IN ('overworld','nether','end'));
ALTER TABLE worlds ADD COLUMN access_policy text NOT NULL DEFAULT 'community'
    CHECK(access_policy IN ('community','participants','isolated'));
UPDATE worlds SET environment=CASE WHEN kind IN ('end','private_end') THEN 'end'
    WHEN kind='nether' THEN 'nether' ELSE 'overworld' END,
    lifetime=CASE WHEN kind='private_end' THEN 'temporary' ELSE 'persistent' END,
    access_policy=CASE WHEN kind='private_end' THEN 'participants'
    WHEN kind='holding' THEN 'isolated' ELSE 'community' END;
