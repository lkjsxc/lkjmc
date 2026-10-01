CREATE TABLE voice_sessions (
    room_id uuid NOT NULL REFERENCES rooms,
    account_id uuid NOT NULL REFERENCES accounts,
    expires_at timestamptz NOT NULL,
    PRIMARY KEY(room_id,account_id)
);
CREATE TABLE observations (
    credential uuid NOT NULL REFERENCES service_credentials,
    observed_at timestamptz NOT NULL DEFAULT now(),
    payload jsonb NOT NULL,
    PRIMARY KEY(credential)
);
CREATE INDEX active_sessions_lease ON game_sessions(lease_until);
CREATE INDEX recent_account_requests ON idempotency(actor,created_at);
CREATE INDEX recent_author_messages ON messages(author,created_at);
CREATE UNIQUE INDEX one_claim_asset_listing ON assets(claim_id) WHERE kind='land' AND state IN ('capturing','escrowed','listed');
