ALTER TABLE game_sessions ADD COLUMN client text NOT NULL DEFAULT 'java' CHECK(client IN ('java','bedrock'));
ALTER TABLE game_sessions ADD COLUMN pending_server_id uuid REFERENCES servers;
ALTER TABLE game_sessions ADD COLUMN route_expires_at timestamptz;
