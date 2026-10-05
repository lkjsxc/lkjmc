-- Membership is scoped to a team; the same account may belong to many teams.
ALTER TABLE team_members DROP CONSTRAINT team_members_account_id_key;
CREATE INDEX team_members_account_team ON team_members(account_id,team_id);

-- No row means personal contributions only. Removing the membership removes
-- this preference without changing the account or another team's membership.
CREATE TABLE team_contribution_selection (
    account_id uuid PRIMARY KEY REFERENCES accounts(id),
    team_id uuid NOT NULL,
    FOREIGN KEY(team_id,account_id) REFERENCES team_members(team_id,account_id)
        ON DELETE CASCADE
);
INSERT INTO team_contribution_selection(account_id,team_id)
SELECT m.account_id,m.team_id FROM team_members m
JOIN teams t ON t.id=m.team_id WHERE t.disbanded_at IS NULL;

-- The recipient is snapshotted at first successful Core acceptance. Keep the
-- historical recipient after the actor leaves; teams are soft-disbanded.
ALTER TABLE game_events ADD COLUMN contribution_team_id uuid REFERENCES teams(id);

CREATE INDEX jobs_operations_history ON jobs(state,created_at DESC,id DESC)
WHERE kind NOT IN ('server.logs','server.files','server.file.read');
