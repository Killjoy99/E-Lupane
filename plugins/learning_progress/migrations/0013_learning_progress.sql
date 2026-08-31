CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS learning_progress (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id      UUID NOT NULL,
    course_id       UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    module_id       UUID,
    lesson_id       UUID,
    content_item_id UUID,
    status          TEXT NOT NULL DEFAULT 'not_started',
    progress_pct    NUMERIC(5,2) NOT NULL DEFAULT 0,
    completed_at    TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_lesson_id  UUID,
    video_timestamp NUMERIC(10,2) DEFAULT 0,
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_learning_progress_student ON learning_progress (student_id, course_id);
CREATE INDEX IF NOT EXISTS idx_learning_progress_course ON learning_progress (course_id);
CREATE INDEX IF NOT EXISTS idx_learning_progress_lesson ON learning_progress (lesson_id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_learning_progress_status'
    ) THEN
        ALTER TABLE learning_progress
            ADD CONSTRAINT chk_learning_progress_status
            CHECK (status IN ('not_started', 'in_progress', 'completed'));
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS progress_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    progress_id     UUID NOT NULL REFERENCES learning_progress(id) ON DELETE CASCADE,
    event_type      TEXT NOT NULL,
    data            JSONB NOT NULL DEFAULT '{}',
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_progress_events_progress ON progress_events (progress_id);
