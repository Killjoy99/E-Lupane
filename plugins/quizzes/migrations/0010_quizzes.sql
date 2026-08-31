CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS quizzes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id       UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    instructions    TEXT NOT NULL DEFAULT '',
    time_limit_minutes INTEGER,
    max_attempts    INTEGER NOT NULL DEFAULT 1,
    passing_score   NUMERIC(5,2) NOT NULL DEFAULT 0,
    randomize_questions BOOLEAN NOT NULL DEFAULT false,
    randomize_answers BOOLEAN NOT NULL DEFAULT false,
    negative_marking BOOLEAN NOT NULL DEFAULT false,
    negative_mark_value NUMERIC(5,2) NOT NULL DEFAULT 0,
    show_results    TEXT NOT NULL DEFAULT 'immediate',
    allow_review    BOOLEAN NOT NULL DEFAULT true,
    status          TEXT NOT NULL DEFAULT 'draft',
    created_by      UUID NOT NULL,
    updated_by      UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_quizzes_course ON quizzes (course_id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_quizzes_status'
    ) THEN
        ALTER TABLE quizzes
            ADD CONSTRAINT chk_quizzes_status
            CHECK (status IN ('draft', 'published', 'archived'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_quizzes_results'
    ) THEN
        ALTER TABLE quizzes
            ADD CONSTRAINT chk_quizzes_results
            CHECK (show_results IN ('immediate', 'delayed', 'never'));
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS quiz_questions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quiz_id         UUID NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    question_type   TEXT NOT NULL,
    title           TEXT NOT NULL,
    body            TEXT NOT NULL DEFAULT '',
    points          NUMERIC(10,2) NOT NULL DEFAULT 1,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    correct_answer  JSONB,
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_by      UUID NOT NULL,
    updated_by      UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_quiz_questions_quiz ON quiz_questions (quiz_id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_quiz_questions_type'
    ) THEN
        ALTER TABLE quiz_questions
            ADD CONSTRAINT chk_quiz_questions_type
            CHECK (question_type IN ('multiple_choice', 'multiple_answer', 'true_false', 'short_answer', 'long_answer', 'fill_blank', 'matching', 'ordering', 'numerical'));
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS quiz_question_options (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id     UUID NOT NULL REFERENCES quiz_questions(id) ON DELETE CASCADE,
    label           TEXT NOT NULL DEFAULT '',
    value           TEXT NOT NULL,
    is_correct      BOOLEAN NOT NULL DEFAULT false,
    sort_order      INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_quiz_question_options_question ON quiz_question_options (question_id);

CREATE TABLE IF NOT EXISTS quiz_attempts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quiz_id         UUID NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    student_id      UUID NOT NULL,
    attempt_number  INTEGER NOT NULL DEFAULT 1,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    submitted_at    TIMESTAMPTZ,
    time_expired_at TIMESTAMPTZ,
    score           NUMERIC(10,2),
    max_score       NUMERIC(10,2),
    passed          BOOLEAN,
    graded          BOOLEAN NOT NULL DEFAULT false,
    status          TEXT NOT NULL DEFAULT 'in_progress'
);

CREATE INDEX IF NOT EXISTS idx_quiz_attempts_quiz_student ON quiz_attempts (quiz_id, student_id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_quiz_attempts_status'
    ) THEN
        ALTER TABLE quiz_attempts
            ADD CONSTRAINT chk_quiz_attempts_status
            CHECK (status IN ('in_progress', 'submitted', 'timed_out', 'graded'));
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS attempt_answers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attempt_id      UUID NOT NULL REFERENCES quiz_attempts(id) ON DELETE CASCADE,
    question_id     UUID NOT NULL REFERENCES quiz_questions(id) ON DELETE CASCADE,
    answer          JSONB NOT NULL DEFAULT '{}',
    is_correct      BOOLEAN,
    points_awarded  NUMERIC(10,2),
    graded_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_attempt_answers_attempt ON attempt_answers (attempt_id);

CREATE TABLE IF NOT EXISTS quiz_assignments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quiz_id         UUID NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    course_id       UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    available_from  TIMESTAMPTZ,
    available_until TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_quiz_assignments_quiz ON quiz_assignments (quiz_id);
CREATE INDEX IF NOT EXISTS idx_quiz_assignments_course ON quiz_assignments (course_id);
