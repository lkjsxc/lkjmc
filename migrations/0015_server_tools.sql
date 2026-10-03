-- Explicit operator intent lives in durable jobs. Never turn legacy host roles
-- into native game OP, nor upgrade membership permissions during this migration.
CREATE INDEX server_passive_retention ON jobs(updated_at DESC)
    WHERE kind IN ('server.logs','server.files','server.file.read');
CREATE INDEX server_operator_member ON jobs(server_id,(payload->>'member'),created_at DESC)
    WHERE kind='server.operator';
