CREATE TABLE adventure_participants (
    adventure_id uuid NOT NULL REFERENCES adventures,
    account_id uuid NOT NULL REFERENCES accounts,
    released_at timestamptz,
    PRIMARY KEY(adventure_id,account_id)
);
CREATE UNIQUE INDEX one_active_adventure_per_player ON adventure_participants(account_id) WHERE released_at IS NULL;
INSERT INTO adventure_participants(adventure_id,account_id,released_at)
SELECT a.id,a.owner,CASE WHEN a.state IN ('closed','refunded') THEN now() ELSE NULL END FROM adventures a;
INSERT INTO adventure_participants(adventure_id,account_id,released_at)
SELECT a.id,m.account_id,CASE WHEN a.state IN ('closed','refunded') THEN now() ELSE NULL END
FROM adventures a JOIN party_members m ON m.party_id=a.party_id WHERE m.account_id<>a.owner;
