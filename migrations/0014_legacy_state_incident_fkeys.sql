-- The legacy state tables precede incidents in migration order. Add their
-- references once the incidents table exists.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'freeze_state_incident_id_fkey') THEN
        ALTER TABLE freeze_state ADD CONSTRAINT freeze_state_incident_id_fkey FOREIGN KEY (incident_id) REFERENCES incidents(id);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'bypass_state_incident_id_fkey') THEN
        ALTER TABLE bypass_state ADD CONSTRAINT bypass_state_incident_id_fkey FOREIGN KEY (incident_id) REFERENCES incidents(id);
    END IF;
END $$;
