-- Reconnecting or linking identities must not clear an existing 30-second PvP window.
ALTER TABLE accounts ADD COLUMN combat_until timestamptz;
UPDATE accounts a SET combat_until=g.combat_until FROM game_sessions g
WHERE g.account_id=a.id AND g.combat_until>now();
