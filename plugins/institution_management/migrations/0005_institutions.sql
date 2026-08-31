-- Institution management plugin: institutions and campuses.
--
-- Version 0005 in the shared `_sqlx_migrations` sequence (auth 0001, assignments
-- 0002, course_management 0003, enrollments 0004). Applied with idempotent raw
-- SQL so it is safe to run repeatedly and alongside the other plugins.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Institutions (universities / colleges). Top of the academic hierarchy.
CREATE TABLE IF NOT EXISTS institutions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          TEXT NOT NULL,
    code          TEXT NOT NULL UNIQUE,
    short_name    TEXT NOT NULL DEFAULT '',
    description   TEXT NOT NULL DEFAULT '',
    logo_url      TEXT NOT NULL DEFAULT '',
    website       TEXT NOT NULL DEFAULT '',
    contact_email TEXT NOT NULL DEFAULT '',
    status        TEXT NOT NULL DEFAULT 'active',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_institutions_status ON institutions (status);
CREATE INDEX IF NOT EXISTS idx_institutions_code ON institutions (code);

-- Campuses: physical locations that belong to an institution.
CREATE TABLE IF NOT EXISTS campuses (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id UUID NOT NULL REFERENCES institutions(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    code           TEXT NOT NULL UNIQUE,
    city           TEXT NOT NULL DEFAULT '',
    country        TEXT NOT NULL DEFAULT '',
    address        TEXT NOT NULL DEFAULT '',
    status         TEXT NOT NULL DEFAULT 'active',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_campuses_institution ON campuses (institution_id);
CREATE INDEX IF NOT EXISTS idx_campuses_status ON campuses (status);
