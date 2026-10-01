-- Trusted adapters may replay a saved event after a player has disconnected.
-- History bounds that replay to an authenticated session in this official server.
CREATE TABLE game_session_history (
    session_id uuid NOT NULL,
    server_id uuid NOT NULL REFERENCES servers,
    account_id uuid NOT NULL REFERENCES accounts,
    profile_id uuid NOT NULL REFERENCES profiles,
    started_at timestamptz NOT NULL DEFAULT now(),
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(session_id,server_id)
);
