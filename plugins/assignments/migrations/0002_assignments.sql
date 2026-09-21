-- Assignments plugin: assignments, resources, submissions, grades, rubrics.
--
-- NOTE: this is version 0002 because the auth plugin owns 0001 in the same
-- `_sqlx_migrations` table of the shared `elupane` database.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Assignments (created by instructors/lecturers).
CREATE TABLE IF NOT EXISTS assignments (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title                   TEXT NOT NULL,
    instructions            TEXT NOT NULL DEFAULT '',
    course_id               UUID,
    created_by              UUID NOT NULL,
    deadline                TIMESTAMPTZ,
    max_marks               NUMERIC(10,2) NOT NULL DEFAULT 100,
    allowed_attempts        INT NOT NULL DEFAULT 1,
    submission_type         TEXT NOT NULL DEFAULT 'any',
    late_submission_policy  TEXT NOT NULL DEFAULT 'allowed_with_penalty',
    late_penalty_percent    NUMERIC(5,2) NOT NULL DEFAULT 0,
    status                  TEXT NOT NULL DEFAULT 'published',
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_assignments_course ON assignments (course_id);
CREATE INDEX IF NOT EXISTS idx_assignments_created_by ON assignments (created_by);

-- Instructor-attached resources (files). Only the server-generated
-- `stored_name` is ever used as a filesystem path.
CREATE TABLE IF NOT EXISTS assignment_resources (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id  UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    stored_name    TEXT NOT NULL,
    mime_type      TEXT NOT NULL DEFAULT 'application/octet-stream',
    size_bytes     BIGINT NOT NULL DEFAULT 0,
    content_hash   TEXT,
    created_by     UUID NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_resources_assignment ON assignment_resources (assignment_id);

-- Submissions (one row per attempt; resubmission = a new attempt).
CREATE TABLE IF NOT EXISTS submissions (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id  UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    student_id     UUID NOT NULL,
    attempt        INT NOT NULL DEFAULT 1,
    text_content   TEXT,
    submitted_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_late        BOOLEAN NOT NULL DEFAULT FALSE,
    status         TEXT NOT NULL DEFAULT 'submitted',
    graded         BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (assignment_id, student_id, attempt)
);

CREATE INDEX IF NOT EXISTS idx_submissions_assignment ON submissions (assignment_id);
CREATE INDEX IF NOT EXISTS idx_submissions_student ON submissions (student_id);

-- Files attached to a submission.
CREATE TABLE IF NOT EXISTS submission_files (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id  UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    stored_name    TEXT NOT NULL,
    mime_type      TEXT NOT NULL DEFAULT 'application/octet-stream',
    size_bytes     BIGINT NOT NULL DEFAULT 0,
    content_hash   TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_submission_files_sub ON submission_files (submission_id);

-- Links attached to a submission.
CREATE TABLE IF NOT EXISTS submission_links (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id  UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    url            TEXT NOT NULL,
    title          TEXT NOT NULL DEFAULT '',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_submission_links_sub ON submission_links (submission_id);

-- Grades (current grade per submission; final = raw minus late penalty).
CREATE TABLE IF NOT EXISTS grades (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id  UUID NOT NULL UNIQUE REFERENCES submissions(id) ON DELETE CASCADE,
    assignment_id  UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    student_id     UUID NOT NULL,
    grader_id      UUID NOT NULL,
    marks          NUMERIC(10,2) NOT NULL,
    raw_marks      NUMERIC(10,2) NOT NULL,
    comments       TEXT NOT NULL DEFAULT '',
    feedback       TEXT NOT NULL DEFAULT '',
    graded_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (assignment_id, student_id)
);

CREATE INDEX IF NOT EXISTS idx_grades_student ON grades (student_id);

-- Grade history (append-only audit trail of every grading event).
CREATE TABLE IF NOT EXISTS grade_history (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id  UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    grader_id      UUID NOT NULL,
    marks          NUMERIC(10,2) NOT NULL,
    raw_marks      NUMERIC(10,2) NOT NULL,
    comments       TEXT NOT NULL DEFAULT '',
    feedback       TEXT NOT NULL DEFAULT '',
    graded_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_grade_history_sub ON grade_history (submission_id);

-- Rubrics (criteria stored as JSON: [{ name, max_marks, description }]).
CREATE TABLE IF NOT EXISTS rubrics (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id  UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    description    TEXT NOT NULL DEFAULT '',
    criteria       JSONB NOT NULL DEFAULT '[]',
    created_by     UUID NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_rubrics_assignment ON rubrics (assignment_id);

-- Marks are modelled as f64 throughout the Rust code (sqlx maps f64 to
-- DOUBLE PRECISION / FLOAT8). Older databases created these columns as NUMERIC,
-- which sqlx refuses to decode into f64. Convert any still-NUMERIC mark
-- columns to DOUBLE PRECISION. Guarded + idempotent: after the first run the
-- data_type is already 'double precision' and the block is a no-op.
DO $$
DECLARE
    r record;
BEGIN
    FOR r IN
        SELECT table_name, column_name
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND data_type = 'numeric'
          AND (table_name, column_name) IN (
              ('assignments', 'max_marks'),
              ('assignments', 'late_penalty_percent'),
              ('grades', 'marks'),
              ('grades', 'raw_marks'),
              ('grade_history', 'marks'),
              ('grade_history', 'raw_marks')
          )
    LOOP
        EXECUTE format('ALTER TABLE %I ALTER COLUMN %I TYPE DOUBLE PRECISION',
                       r.table_name, r.column_name);
    END LOOP;
END $$;

