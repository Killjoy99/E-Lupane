-- Academics plugin: academic structure and curriculum.
--
-- Version 0006 in the shared `_sqlx_migrations` sequence (auth 0001, assignments
-- 0002, course_management 0003, enrollments 0004, institution_management 0005).
-- Applied with idempotent raw SQL so it is safe to run repeatedly.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Faculties belong to an institution (from the institution_management plugin).
CREATE TABLE IF NOT EXISTS faculties (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id UUID NOT NULL,
    name           TEXT NOT NULL,
    code           TEXT NOT NULL UNIQUE,
    description    TEXT NOT NULL DEFAULT '',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_faculties_institution ON faculties (institution_id);

-- Departments belong to a faculty.
CREATE TABLE IF NOT EXISTS departments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    faculty_id  UUID NOT NULL,
    name        TEXT NOT NULL,
    code        TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    head_id     UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_departments_faculty ON departments (faculty_id);

-- Programs belong to a department.
CREATE TABLE IF NOT EXISTS programs (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    department_id  UUID NOT NULL,
    name           TEXT NOT NULL,
    code           TEXT NOT NULL UNIQUE,
    degree_type    TEXT NOT NULL DEFAULT 'degree',
    duration_years INTEGER NOT NULL DEFAULT 4,
    description    TEXT NOT NULL DEFAULT '',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_programs_department ON programs (department_id);

-- Academic years.
CREATE TABLE IF NOT EXISTS academic_years (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    code       TEXT NOT NULL UNIQUE,
    start_date DATE NOT NULL,
    end_date   DATE NOT NULL,
    status     TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Semesters belong to an academic year.
CREATE TABLE IF NOT EXISTS semesters (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    academic_year_id UUID NOT NULL,
    name             TEXT NOT NULL,
    code             TEXT NOT NULL UNIQUE,
    session_number   INTEGER NOT NULL DEFAULT 1,
    start_date       DATE NOT NULL,
    end_date         DATE NOT NULL,
    status           TEXT NOT NULL DEFAULT 'planned',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_semesters_year ON semesters (academic_year_id);

-- Classes / cohorts: a group of students on a program in a given academic year.
CREATE TABLE IF NOT EXISTS classes (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    program_id       UUID NOT NULL,
    academic_year_id UUID NOT NULL,
    name             TEXT NOT NULL,
    code             TEXT NOT NULL UNIQUE,
    intake_date      DATE,
    capacity         INTEGER NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_classes_program ON classes (program_id);
CREATE INDEX IF NOT EXISTS idx_classes_year ON classes (academic_year_id);

-- Course offerings: a course (course_management) taught in a semester.
-- NOTE: `courses.id` is created by the course_management plugin (0003). The FK
-- is added with a guarded block so plugin startup order does not matter.
CREATE TABLE IF NOT EXISTS course_offerings (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id     UUID NOT NULL,
    semester_id   UUID NOT NULL,
    program_id    UUID NOT NULL,
    instructor_id UUID NOT NULL,
    section       TEXT NOT NULL DEFAULT '',
    max_enrolment INTEGER NOT NULL DEFAULT 0,
    status        TEXT NOT NULL DEFAULT 'open',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_offerings_course ON course_offerings (course_id);
CREATE INDEX IF NOT EXISTS idx_offerings_semester ON course_offerings (semester_id);
CREATE INDEX IF NOT EXISTS idx_offerings_program ON course_offerings (program_id);

DO $$
BEGIN
    IF to_regclass('public.semesters') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_offerings_semester'
       ) THEN
        ALTER TABLE course_offerings
            ADD CONSTRAINT fk_offerings_semester
            FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE CASCADE;
    END IF;

    IF to_regclass('public.programs') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_offerings_program'
       ) THEN
        ALTER TABLE course_offerings
            ADD CONSTRAINT fk_offerings_program
            FOREIGN KEY (program_id) REFERENCES programs(id) ON DELETE CASCADE;
    END IF;

    IF to_regclass('public.courses') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_offerings_course'
       ) THEN
        ALTER TABLE course_offerings
            ADD CONSTRAINT fk_offerings_course
            FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE;
    END IF;
END $$;

