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

-- Built-in achievements are identified by their immutable seed key AND original
-- seed text. Administrator-authored replacements are intentionally left alone.
ALTER TABLE achievements ADD COLUMN title_message jsonb;
ALTER TABLE achievements ADD COLUMN description_message jsonb;
UPDATE achievements a SET
    title_message=jsonb_build_object('id','achievement.'||a.key||'.title','params','{}'::jsonb),
    description_message=jsonb_build_object('id','achievement.'||a.key||'.description','params','{}'::jsonb)
FROM (VALUES
    ('first_claim','暮らしのはじまり','最初の土地を保護する'),
    ('builder_256','小さな家から','自分でブロックを256個置く'),
    ('builder_2048','まちづくり','自分でブロックを2,048個置く'),
    ('explorer_10000','遠くの景色','徒歩で10,000ブロック進む'),
    ('farmer_512','畑と暮らす','作物を512個収穫する'),
    ('team_builder_4096','みんなの拠点','チームでブロックを4,096個置く'),
    ('team_adventure_3','冒険仲間','チームで一時 End の目標を3回達成する')
) AS seed(key,title,description)
WHERE a.key=seed.key AND a.title=seed.title AND a.description=seed.description;
ALTER TABLE trust_ranks ADD COLUMN name_message jsonb;
UPDATE trust_ranks SET name_message='{"id":"system.player_rank","params":{}}'::jsonb
WHERE id=0 AND name='プレイヤー';
ALTER TABLE principals ADD COLUMN name_message jsonb;
UPDATE principals SET name_message='{"id":"system.transaction_fees","params":{}}'::jsonb
WHERE id='00000000-0000-0000-0000-000000000001' AND kind='system' AND name='取引手数料';
