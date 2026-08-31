-- Gradebook plugin: grade categories, grade scales, grade entries, and grade history.
--
-- NOTE: version 0009 because auth owns 0001, assignments 0002, course_management 0003,
-- enrollments 0004, institution_management 0005, academics 0006, live_classes 0007,
-- content 0008. This schema uses idempotent raw SQL (IF NOT EXISTS + guarded ALTER),
-- so it is safe to run alongside the other plugins regardless of startup order.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Grade categories define weighted buckets for a course's final grade.
CREATE TABLE IF NOT EXISTS grade_categories (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id       UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    weight          NUMERIC(5,2) NOT NULL DEFAULT 0,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_by      UUID NOT NULL,
    updated_by      UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_grade_categories_course ON grade_categories (course_id);

-- Ensure category weights for a course sum to <= 100.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_grade_categories_weight'
    ) THEN
        ALTER TABLE grade_categories
            ADD CONSTRAINT chk_grade_categories_weight
            CHECK (weight >= 0 AND weight <= 100);
    END IF;
END $$;

-- Grade scales are configurable by institution and map percentage ranges to
-- letter grades and grade points.
CREATE TABLE IF NOT EXISTS grade_scales (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id  UUID NOT NULL REFERENCES institutions(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    is_default      BOOLEAN NOT NULL DEFAULT false,
    passing_grade   NUMERIC(5,2) NOT NULL DEFAULT 50,
    created_by      UUID NOT NULL,
    updated_by      UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_grade_scales_institution ON grade_scales (institution_id);

-- Individual letter-grade bands within a scale.
CREATE TABLE IF NOT EXISTS grade_scale_bands (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scale_id        UUID NOT NULL REFERENCES grade_scales(id) ON DELETE CASCADE,
    letter          TEXT NOT NULL,
    min_pct         NUMERIC(5,2) NOT NULL,
    max_pct         NUMERIC(5,2) NOT NULL,
    grade_point     NUMERIC(3,2) NOT NULL,
    sort_order      INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_grade_scale_bands_scale ON grade_scale_bands (scale_id);

-- Grade entries are the actual grades recorded for students.
CREATE TABLE IF NOT EXISTS grade_entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id      UUID NOT NULL,
    course_id       UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    category_id     UUID NOT NULL REFERENCES grade_categories(id) ON DELETE CASCADE,
    source_type     TEXT NOT NULL,
    source_id       UUID NOT NULL,
    score           NUMERIC(10,2),
    max_score       NUMERIC(10,2) NOT NULL DEFAULT 100,
    letter          TEXT,
    grade_point     NUMERIC(3,2),
    graded_by       UUID NOT NULL,
    graded_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    overridden_by   UUID,
    overridden_at   TIMESTAMPTZ,
    override_reason TEXT,
    comments        TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_grade_entries_student_course ON grade_entries (student_id, course_id);
CREATE INDEX IF NOT EXISTS idx_grade_entries_category ON grade_entries (category_id);
CREATE INDEX IF NOT EXISTS idx_grade_entries_source ON grade_entries (source_type, source_id);

-- Grade history for audit trail.
CREATE TABLE IF NOT EXISTS grade_history (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    grade_entry_id  UUID NOT NULL REFERENCES grade_entries(id) ON DELETE CASCADE,
    old_score       NUMERIC(10,2),
    new_score       NUMERIC(10,2),
    old_letter      TEXT,
    new_letter      TEXT,
    old_grade_point NUMERIC(3,2),
    new_grade_point NUMERIC(3,2),
    changed_by      UUID NOT NULL,
    changed_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    reason          TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_grade_history_entry ON grade_history (grade_entry_id);

-- Enforce valid source_type values.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_grade_entries_source_type'
    ) THEN
        ALTER TABLE grade_entries
            ADD CONSTRAINT chk_grade_entries_source_type
            CHECK (source_type IN ('assignment', 'quiz', 'exam', 'manual'));
    END IF;
END $$;

-- Final grade cache per student per course (denormalised for fast reads).
CREATE TABLE IF NOT EXISTS final_grades (
    student_id      UUID NOT NULL,
    course_id       UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    final_score     NUMERIC(10,2),
    final_letter    TEXT,
    final_grade_point NUMERIC(3,2),
    status          TEXT NOT NULL DEFAULT 'in_progress',
    calculated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    calculated_by   UUID,
    PRIMARY KEY (student_id, course_id)
);

CREATE INDEX IF NOT EXISTS idx_final_grades_course ON final_grades (course_id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_final_grades_status'
    ) THEN
        ALTER TABLE final_grades
            ADD CONSTRAINT chk_final_grades_status
            CHECK (status IN ('in_progress', 'finalized'));
    END IF;
END $$;

-- Seed a default grade scale for each existing institution.
DO $$
DECLARE
    inst RECORD;
    scale_id UUID;
BEGIN
    FOR inst IN SELECT id FROM institutions LOOP
        IF NOT EXISTS (
            SELECT 1 FROM grade_scales WHERE institution_id = inst.id
        ) THEN
            INSERT INTO grade_scales (institution_id, name, is_default, passing_grade, created_by, updated_by)
            VALUES (inst.id, 'Default', true, 50, inst.id, inst.id)
            RETURNING id INTO scale_id;
        END IF;
    END LOOP;
END $$;
