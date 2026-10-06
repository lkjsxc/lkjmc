-- Existing requests retain their original requester-to-recipient direction.
-- Keep the pending requester/recipient identity and unique index unchanged.
ALTER TABLE invitations ADD COLUMN teleport_here boolean NOT NULL DEFAULT false;
ALTER TABLE invitations ADD CONSTRAINT teleport_direction_kind
    CHECK (NOT teleport_here OR kind='teleport');
