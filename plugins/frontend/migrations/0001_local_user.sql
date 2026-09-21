-- Google OAuth-backed platform users ("Sign in with Google").
--
-- This table is owned by the frontend server (not the auth plugin). It is
-- created idempotently at boot by `google_classroom_bridge::ensure_table`,
-- which runs this same DDL; the file exists for reference / manual setup.
--
-- Roles come from the .env permission matrix on every sign-in:
--   GOOGLE_CLASSROOM_SUPER_ADMINS → ADMINS → DEVELOPERS → LECTURERS,
-- defaulting to 'student'.

CREATE TABLE IF NOT EXISTS local_user (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       TEXT NOT NULL UNIQUE,
    username    TEXT NOT NULL UNIQUE,
    full_name   TEXT NOT NULL,
    picture     TEXT,
    roles       TEXT[] NOT NULL DEFAULT ARRAY['student']::TEXT[],
    active      BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_local_user_email ON local_user (email);

-- Tables created before the 1:1 username guard: add it idempotently.
DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'local_user_username_key'
    ) THEN
        ALTER TABLE local_user ADD CONSTRAINT local_user_username_key UNIQUE (username);
    END IF;
END $$;
