-- First-party generated text has explicit provenance; player titles stay verbatim.
ALTER TABLE assets ADD COLUMN title_message jsonb;
ALTER TABLE assets ADD CONSTRAINT assets_title_message_contract CHECK (
    title_message IS NULL OR (
        jsonb_typeof(title_message)='object'
        AND jsonb_typeof(title_message->'id')='string'
        AND jsonb_typeof(title_message->'params')='object'
        AND title_message ? 'id' AND title_message ? 'params'
    )
);

UPDATE assets a SET title_message='{"id":"system.expedition_refund_items","params":{}}'::jsonb
FROM jobs j, adventures e
WHERE a.job_id=j.id AND j.kind='adventure.cancel' AND e.material_asset=a.id
  AND a.title='冒険準備の返却：エンダーアイ12個';

-- These exact strings were written exclusively by Core settlement. Arbitrary
-- historical diagnostics remain stored for administrators and project generically.
UPDATE servers SET error='{"id":"system.server_operation_failed","params":{}}'
WHERE error='操作に失敗しました。ジョブの詳細を確認してください。';
UPDATE backups SET error='{"id":"system.backup_failed","params":{}}'
WHERE error='保存に失敗しました。ジョブを確認してください。';
UPDATE notifications SET body=jsonb_set(body,'{message}',
    '{"id":"text.collect_your_12_eyes_of_ender_from_stored_assets","params":{}}'::jsonb)
WHERE kind='adventure_refund' AND body->>'message'='Collect your 12 Eyes of Ender from stored assets.';
