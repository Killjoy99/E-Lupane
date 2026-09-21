-- 0008: add the `expired` status.
--
-- A scheduled session whose `scheduled_end` has passed without the host
-- starting it is expired rather than left joinable forever (or never
-- terminated). The plugin sweeps these lazily on read/join (see
-- `repository::expire_stale_sessions`) — no background worker needed.
--
-- Idempotent: re-create the CHECK constraint only when `expired` is missing.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'live_sessions_status_check'
          AND conrelid = 'live_sessions'::regclass
    ) AND NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'live_sessions_status_check'
          AND conrelid = 'live_sessions'::regclass
          AND pg_get_constraintdef(oid) LIKE '%expired%'
    ) THEN
        ALTER TABLE live_sessions DROP CONSTRAINT live_sessions_status_check;
        ALTER TABLE live_sessions ADD CONSTRAINT live_sessions_status_check
            CHECK (status IN ('scheduled','live','ended','cancelled','expired'));
    END IF;
END $$;
