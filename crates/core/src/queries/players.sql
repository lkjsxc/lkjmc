-- One authorized snapshot. Suggestions reveal identities, not server IDs,
-- coordinates, session IDs, or an administrator-only view of private presence.
WITH current_server AS (
  SELECT g.server_id
  FROM game_sessions g
  JOIN profiles p ON p.id=g.profile_id AND p.account_id=g.account_id
    AND p.status='active' AND p.native_uuid=g.native_uuid
  JOIN servers s ON s.id=g.server_id
  WHERE g.account_id=$1 AND g.lease_until>now()
    AND (s.visibility='public' OR s.owner=$1
      OR EXISTS(SELECT 1 FROM accounts WHERE id=$1 AND administrator)
      OR EXISTS(SELECT 1 FROM server_members WHERE server_id=s.id AND account_id=$1)
      OR EXISTS(SELECT 1 FROM community_members WHERE community_id=s.community_id AND account_id=$1))
), candidates AS (
  SELECT a.id,p.name,r.name AS rank,r.name_message AS rank_message,
    a.activity_policy<>'none' AND EXISTS(
      SELECT 1 FROM game_sessions g JOIN profiles gp ON gp.id=g.profile_id
        AND gp.account_id=g.account_id AND gp.status='active' AND gp.native_uuid=g.native_uuid
      WHERE g.account_id=a.id AND g.lease_until>now()
        AND g.server_id IN (SELECT server_id FROM current_server)
    ) AS same_server,
    EXISTS(SELECT 1 FROM party_members mine JOIN party_members other USING(party_id)
      JOIN parties party ON party.id=mine.party_id AND party.closed_at IS NULL
      WHERE mine.account_id=$1 AND other.account_id=a.id) AS shared_party,
    EXISTS(SELECT 1 FROM team_members mine JOIN team_members other USING(team_id)
      JOIN teams team ON team.id=mine.team_id AND team.disbanded_at IS NULL
      WHERE mine.account_id=$1 AND other.account_id=a.id) AS shared_team,
    EXISTS(SELECT 1 FROM friendships f WHERE f.first_id=least($1,a.id)
      AND f.second_id=greatest($1,a.id) AND f.state='accepted') AS friend
  FROM accounts a JOIN principals p ON p.id=a.id JOIN trust_ranks r ON r.id=a.trust_rank
  WHERE a.id<>$1 AND a.merged_into IS NULL
    AND (a.banned_until IS NULL OR a.banned_until<=now())
    AND EXISTS(SELECT 1 FROM profiles WHERE account_id=a.id AND status='active')
    AND NOT EXISTS(SELECT 1 FROM blocks b WHERE
      (b.actor=$1 AND b.target=a.id) OR (b.actor=a.id AND b.target=$1))
), ordered AS (
  SELECT *,CASE WHEN $2<>'' THEN 4 WHEN same_server THEN 0
    WHEN shared_party THEN 1 WHEN shared_team THEN 2 WHEN friend THEN 3 ELSE 4 END AS priority
  FROM candidates
  WHERE ($2='' AND (same_server OR shared_party OR shared_team OR friend))
    OR ($2<>'' AND (name ILIKE '%'||replace(replace(replace($2,chr(92),chr(92)||chr(92)),'%',chr(92)||'%'),'_',chr(92)||'_')||'%' OR id::text=$2))
  ORDER BY priority,name,id LIMIT 30
)
SELECT coalesce(jsonb_agg(jsonb_build_object('id',id,'name',name,'rank',rank,'rank_message',rank_message)
  ORDER BY priority,name,id),'[]') FROM ordered
