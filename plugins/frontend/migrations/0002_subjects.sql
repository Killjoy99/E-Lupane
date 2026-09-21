-- Subjects: first-class "subject/module" units under a course (the admin-created
-- degree-like course). A course has subjects, a subject carries content, assignments
-- and live classes (re-pointed via subject_id on the existing learning-object tables).
--
-- Students enrolled in a course automatically inherit access to its published
-- subjects; additionally a subject invite link grants direct access to that subject
-- (recorded in subject_access with source = 'invite').

CREATE TABLE IF NOT EXISTS subjects (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id   UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    code        TEXT NOT NULL DEFAULT '',
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    sort_order  INTEGER NOT NULL DEFAULT 0,
    status      TEXT NOT NULL DEFAULT 'draft'
                CHECK (status IN ('draft','published','archived')),
    created_by  UUID,
    updated_by  UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_subjects_course_code UNIQUE (course_id, code)
);

CREATE INDEX IF NOT EXISTS idx_subjects_course ON subjects (course_id);
CREATE INDEX IF NOT EXISTS idx_subjects_status ON subjects (status);

-- Direct student→subject access (source = invite for link-joined students).
-- Course-enrollment inheritance is computed on the fly (active course_enrollments
-- → all published subjects of that course), so it needs no rows here.

CREATE TABLE IF NOT EXISTS subject_access (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id  UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    student_id  UUID NOT NULL,
    source      TEXT NOT NULL DEFAULT 'invite'
                CHECK (source IN ('inherit','invite')),
    created_by  UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_subject_student UNIQUE (subject_id, student_id)
);

CREATE INDEX IF NOT EXISTS idx_subject_access_student ON subject_access (student_id);
CREATE INDEX IF NOT EXISTS idx_subject_access_subject ON subject_access (subject_id);

-- Subject invite links: one token grants access to one subject.

CREATE TABLE IF NOT EXISTS subject_invites (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id  UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    token      TEXT NOT NULL UNIQUE,
    created_by  UUID,
    max_uses   INTEGER,
    uses       INTEGER NOT NULL DEFAULT 0,
    expires_at  TIMESTAMPTZ,
    active     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_subject_invites_subject ON subject_invites (subject_id);
CREATE INDEX IF NOT EXISTS idx_subject_invites_token ON subject_invites (token);

-- Re-point learning objects at their subject (content modules, assignments, live sessions).
-- Columns are nullable so legacy course-scoped rows keep working; new rows created
-- from/within a subject set subject_id. Guarded so plugin startup order does not matter.


DO $SUBJ$
BEGIN
    IF to_regclass('public.content_modules') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_name = 'content_modules' AND column_name = 'subject_id'
       ) THEN
        ALTER TABLE content_modules ADD COLUMN subject_id UUID;
    END IF;

    IF to_regclass('public.assignments') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_name = 'assignments' AND column_name = 'subject_id'
       ) THEN
        ALTER TABLE assignments ADD COLUMN subject_id UUID;
    END IF;

    IF to_regclass('public.live_sessions') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_name = 'live_sessions' AND column_name = 'subject_id'
       ) THEN
        ALTER TABLE live_sessions ADD COLUMN subject_id UUID;
    END IF;
END $SUBJ$;

CREATE INDEX IF NOT EXISTS idx_content_modules_subject ON content_modules (subject_id);
CREATE INDEX IF NOT EXISTS idx_assignments_subject ON assignments (subject_id);
CREATE INDEX IF NOT EXISTS idx_live_sessions_subject ON live_sessions (subject_id);