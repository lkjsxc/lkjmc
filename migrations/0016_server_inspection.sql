-- A file maintenance window is independent of the desired Minecraft state.
ALTER TABLE servers ADD COLUMN inspection jsonb;
CREATE FUNCTION freeze_server_inspection() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  IF OLD.inspection IS NOT NULL AND NEW.inspection IS NOT NULL AND
     (NEW.desired <> 'stopped' OR NEW.memory_mib <> OLD.memory_mib OR
      NEW.cpu_millis <> OLD.cpu_millis OR NEW.storage_mib <> OLD.storage_mib) THEN
    RAISE EXCEPTION 'Close file inspection before changing the game target or resources';
  END IF;
  IF NEW.inspection IS NOT NULL THEN NEW.maintenance := true; END IF;
  RETURN NEW;
END $$;
CREATE TRIGGER server_inspection_freeze BEFORE UPDATE ON servers
FOR EACH ROW EXECUTE FUNCTION freeze_server_inspection();
CREATE INDEX server_inspection_jobs ON jobs(server_id,created_at)
WHERE kind='server.inspection';
