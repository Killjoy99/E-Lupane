-- Course management plugin: courses, instructor authorization.
--
-- NOTE: this is version 0003 because the auth plugin owns 0001 and the
-- assignments plugin owns 0002 in the same `_sqlx_migrations` table of the
-- shared `elupane` database. Like the other plugins, this schema is applied
-- with idempotent raw SQL (never `sqlx::migrate!`).

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Courses (created and managed by instructors).
CREATE TABLE IF NOT EXISTS courses (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code                  TEXT NOT NULL UNIQUE,
    title                 TEXT NOT NULL,
    short_description     TEXT NOT NULL DEFAULT '',
    description           TEXT NOT NULL DEFAULT '',
    thumbnail             TEXT NOT NULL DEFAULT '',
    banner                TEXT NOT NULL DEFAULT '',
    category              TEXT NOT NULL DEFAULT '',
    subject               TEXT NOT NULL DEFAULT '',
    level                 TEXT NOT NULL DEFAULT '',
    language              TEXT NOT NULL DEFAULT 'en',
    difficulty            TEXT NOT NULL DEFAULT 'beginner',
    duration              TEXT NOT NULL DEFAULT '',
    learning_objectives   JSONB NOT NULL DEFAULT '[]',
    target_audience       JSONB NOT NULL DEFAULT '[]',
    prerequisites         JSONB NOT NULL DEFAULT '[]',
    tags                  JSONB NOT NULL DEFAULT '[]',
    status                TEXT NOT NULL DEFAULT 'draft',
    owner_id              UUID NOT NULL,
    created_by            UUID NOT NULL,
    updated_by            UUID,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Drop the legacy `price` column (courses are free on the platform).
ALTER TABLE courses DROP COLUMN IF EXISTS price;

CREATE INDEX IF NOT EXISTS idx_courses_owner ON courses (owner_id);
CREATE INDEX IF NOT EXISTS idx_courses_status ON courses (status);
CREATE INDEX IF NOT EXISTS idx_courses_code ON courses (code);

-- Instructors authorised to manage a course (the owner is implicitly
-- authorised; this table records additional instructors).
CREATE TABLE IF NOT EXISTS course_instructors (
    course_id   UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL,
    added_by    UUID NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (course_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_course_instructors_user ON course_instructors (user_id);

-- Link assignments to courses. The assignments plugin stores a plain
-- `course_id` UUID; once the courses table exists we promote that into a real
-- foreign key. Guarded so this works regardless of plugin startup order and is
-- safe to re-run.
DO $$
BEGIN
    IF to_regclass('public.assignments') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_assignments_course'
       ) THEN
        ALTER TABLE assignments
            ADD CONSTRAINT fk_assignments_course
            FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Ensure course-linked lessons/sessions remain anchored to the same course
-- registry. This is the same pattern used for assignments and keeps the data
-- model consistent for course-scoped learning objects.
DO $$
BEGIN
    IF to_regclass('public.live_sessions') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_live_sessions_course'
       ) THEN
        ALTER TABLE live_sessions
            ADD CONSTRAINT fk_live_sessions_course
            FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE;
    END IF;
END $$;
