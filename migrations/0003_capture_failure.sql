-- A failed capture is terminal only when the adapter proves no physical effect
-- or a durable rollback. Such assets must not lock their original plot forever.
ALTER TABLE assets DROP CONSTRAINT assets_state_check;
ALTER TABLE assets ADD CONSTRAINT assets_state_check CHECK
  (state IN ('capturing','escrowed','listed','placing','placed','delivered','quarantined','cancelled'));
